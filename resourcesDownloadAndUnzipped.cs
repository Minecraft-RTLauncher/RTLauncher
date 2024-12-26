using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.IO;
using System.IO.Compression;
using System.Net.Http;
using System.Runtime.Intrinsics.X86;
using System.Security.Cryptography;
using System.Text.Json;
using System.Threading;
using System.Threading.Tasks;

class Program
{
    static string basePath = Path.Combine(Directory.GetCurrentDirectory(), ".minecraft");
    static string assetsDir = Path.Combine(basePath, "assets");
    static int maxRetries = 50;
    static int retryDelayMs = 100;
    static int maxConcurrentLibrariesDownloads = 32;
    static int maxConcurrentAssetsDownloads = 1024;
    static int numDownloadThreads = 16;
    static HttpClient httpClient = new HttpClient();

    static readonly ConcurrentQueue<Exception> libraryExtractionErrors = new ConcurrentQueue<Exception>();
    static readonly ConcurrentQueue<Exception> assetDownloadErrors = new ConcurrentQueue<Exception>();

    static async Task Main(string[] args)
    {
        string version = "1.19";
        await SetupMinecraftDirectory(version);

        LogErrors(libraryExtractionErrors, "Library Extraction Errors");
        LogErrors(assetDownloadErrors, "Asset Download Errors");
    }

    static void LogErrors(ConcurrentQueue<Exception> errorQueue, string errorType)
    {
        while (errorQueue.TryDequeue(out Exception ex))
        {
            Console.WriteLine($"{errorType}: {ex.Message}{Environment.NewLine}{ex.StackTrace}");
        }
    }

    static async Task SetupMinecraftDirectory(string version)
    {
        var versionDir = Path.Combine(basePath, "versions", version);
        var librariesDir = Path.Combine(basePath, "libraries");
        CreateDirectories(versionDir, librariesDir, assetsDir);
        Directory.CreateDirectory(versionDir);

        string versionJsonPath = Path.Combine(versionDir, $"{version}.json");
        await DownloadVersionJson(version, versionJsonPath);

        var versionData = JsonDocument.Parse(await File.ReadAllTextAsync(versionJsonPath));

        await Task.WhenAll(
            HandleClientFile(versionData, versionDir, "Client"),
            HandleLibraries(versionData, librariesDir, "Libraries",version),
            HandleAssetIndex(versionData, versionDir, "Assets", version)
        );
    }

    static void CreateDirectories(params string[] paths)
    {
        foreach (var path in paths)
        {
            Directory.CreateDirectory(path);
        }
    }

    static async Task DownloadVersionJson(string version, string outputPath)
    {
        using (var client = new HttpClient())
        {
            string url = $"https://bmclapi2.bangbang93.com/version/{version}/json";
            var response = await client.GetStringAsync(url);
            await File.WriteAllTextAsync(outputPath, response);
        }
    }

    static async Task HandleClientFile(JsonDocument versionData, string versionDir, string taskName)
    {
        JsonElement downloads = versionData.RootElement.GetProperty("downloads");
        string filePath = Path.Combine(versionDir, "client.jar");
        if (File.Exists(filePath))
        {
            Console.WriteLine($"{taskName} file already exists, skipping download.");
            return;
        }
        await DownloadAndVerify(downloads.GetProperty("client"), filePath, taskName);
    }

    static async Task HandleLibraries(JsonDocument versionData, string librariesDir, string taskName,string version)
    {
        var libraries = versionData.RootElement.GetProperty("libraries");
        var downloadTasks = new List<Task>();
        var semaphore = new SemaphoreSlim(maxConcurrentLibrariesDownloads);

        foreach (var library in libraries.EnumerateArray())
        {
            var downloadInfo = library.GetProperty("downloads").GetProperty("artifact");
            string path = library.GetProperty("downloads").GetProperty("artifact").GetProperty("path").ToString();
            string filePath = Path.Combine(librariesDir, path);
            if (File.Exists(filePath))
            {
                Console.WriteLine($"{taskName} - {path} already exists, skipping download.");
                continue;
            }
            Directory.CreateDirectory(Path.GetDirectoryName(filePath));
            downloadTasks.Add(DownloadAndExtractLibraryWithRateLimit(downloadInfo, filePath, $"{taskName} - {path}", semaphore,version));
        }
        try
        {
            await Task.WhenAll(downloadTasks);
        }
        catch (AggregateException ex)
        {
            Console.WriteLine($"Library download failed: {ex.Message}");
            foreach (var innerEx in ex.InnerExceptions)
            {
                Console.WriteLine($"Inner Exception: {innerEx.Message}");
            }
        }
    }

    static async Task HandleAssetIndex(JsonDocument versionData, string versionDir, string taskName, string version)
    {
        var assetIndex = versionData.RootElement.GetProperty("assetIndex");
        string url = assetIndex.GetProperty("url").GetString();
        string assetIndexFilePath = Path.Combine(versionDir, $"{version}-assets.json");
        if (File.Exists(assetIndexFilePath))
        {
            Console.WriteLine($"{taskName} - Index file already exists, skipping download.");
            return;
        }
        await DownloadFile(url, assetIndexFilePath, $"{taskName} - Index");

        var assetData = JsonDocument.Parse(await File.ReadAllTextAsync(assetIndexFilePath));
        var objects = assetData.RootElement.GetProperty("objects");

        var assetTasks = new List<Task>();
        var semaphore = new SemaphoreSlim(maxConcurrentAssetsDownloads);
        int totalAssets = objects.EnumerateObject().Count();
        int assetsDownloaded = 0;
        var completedAssetsCounter = new Counter();

        Console.WriteLine($"Downloading {totalAssets} assets...");

        foreach (var asset in objects.EnumerateObject())
        {
            string hash = asset.Value.GetProperty("hash").GetString();
            string objectUrl = $"https://resources.download.minecraft.net/{hash.Substring(0, 2)}/{hash}";
            string objectPath = Path.Combine(assetsDir, "objects", hash.Substring(0, 2));
            string fullAssetPath = Path.Combine(objectPath, hash);

            if (File.Exists(fullAssetPath))
            {
                Console.WriteLine($"Asset {hash} already exists, skipping download.");
                completedAssetsCounter.Increment();
                continue;
            }
            Directory.CreateDirectory(objectPath);
            assetTasks.Add(DownloadAssetWithRateLimit(objectUrl, hash, objectPath, semaphore, totalAssets, completedAssetsCounter));
        }

        //Improved Semaphore Handling
        while (assetsDownloaded < totalAssets)
        {
            int remainingAssets = totalAssets - assetsDownloaded;
            int concurrentDownloads = Math.Min(remainingAssets, maxConcurrentAssetsDownloads);
            int acquired = 0;

            while (acquired < concurrentDownloads && semaphore.Wait(0))
            {
                acquired++;
            }

            if (acquired == 0)
            {
                await Task.Delay(retryDelayMs);
                continue;
            }

            try
            {
                await Task.WhenAll(assetTasks.ToArray());
            }
            catch (AggregateException ex)
            {
                Console.WriteLine($"Asset download failed with AggregateException: {ex.Message}");
                foreach (var innerEx in ex.InnerExceptions)
                {
                    assetDownloadErrors.Enqueue(innerEx);
                    Console.WriteLine($"Inner Exception: {innerEx.Message}");
                }
            }
            assetsDownloaded += acquired;
            semaphore.Release(acquired);
        }

        Console.WriteLine("All assets downloaded.");
    }

    static async Task DownloadAssetWithRateLimit(string url, string hash, string objectPath, SemaphoreSlim semaphore, int totalAssets, Counter completedAssetsCounter)
    {
        await semaphore.WaitAsync();
        try
        {
            await DownloadFile(url, Path.Combine(objectPath, hash), hash);
            completedAssetsCounter.Increment();
            Console.WriteLine($"Asset {hash} downloaded. {completedAssetsCounter.Value}/{totalAssets} complete.");
        }
        catch (Exception ex)
        {
            Console.WriteLine($"Error downloading asset {hash}: {ex.Message}");
        }
        finally
        {
            semaphore.Release();
        }
    }

    static async Task DownloadAndVerifyWithRateLimit(JsonElement downloadInfo, string filePath, string taskName, SemaphoreSlim semaphore)
    {
        await semaphore.WaitAsync();
        try
        {
            await DownloadAndVerify(downloadInfo, filePath, taskName);
        }
        finally
        {
            semaphore.Release();
        }
    }

    static async Task DownloadAndVerify(JsonElement downloadInfo, string filePath, string taskName)
    {
        string sha1 = downloadInfo.GetProperty("sha1").GetString();
        string url = downloadInfo.GetProperty("url").GetString();
        await DownloadWithRetry(url, filePath, sha1, taskName);
    }

    static async Task DownloadFile(string url, string filePath, string taskName)
    {
        if (File.Exists(filePath))
        {
            Console.WriteLine($"{taskName} file already exists, skipping download.");
            return;
        }
        await DownloadWithRetry(url, filePath, null, taskName);
    }

    static async Task DownloadWithRetry(string url, string filePath, string sha1, string taskName)
    {
        Console.WriteLine(url);
        int retries = 0;
        while (retries < maxRetries)
        {
            try
            {
                using (var client = new HttpClient())
                {
                    var request = new HttpRequestMessage(HttpMethod.Get, url);
                    var response = await client.SendAsync(request, HttpCompletionOption.ResponseHeadersRead);
                    response.EnsureSuccessStatusCode();
                    long totalBytes = response.Content.Headers.ContentLength ?? 0;
                    using (var fileStream = new FileStream(filePath, FileMode.Create, FileAccess.Write))
                    {
                        using (var contentStream = await response.Content.ReadAsStreamAsync())
                        {
                            byte[] buffer = new byte[81920];
                            int bytesRead;
                            while ((bytesRead = await contentStream.ReadAsync(buffer, 0, buffer.Length)) > 0)
                            {
                                await fileStream.WriteAsync(buffer, 0, bytesRead);
                            }
                        }
                    }
                    if (!string.IsNullOrEmpty(sha1) && !VerifyFileSha1(filePath, sha1))
                    {
                        Console.WriteLine($"{taskName} SHA1 verification failed, retrying...");
                        retries++;
                        await Task.Delay(retryDelayMs);
                        continue;
                    }
                    Console.WriteLine($"{taskName} 下载完成!");
                    return;
                }
            }
            catch (HttpRequestException ex)
            {
                Console.WriteLine($"{taskName} 下载失败 (HTTP Error): {ex.Message}, Retrying in {retryDelayMs}ms...");
                if (ex.InnerException != null)
                {
                    Console.WriteLine($"Inner Exception: {ex.InnerException.Message}");
                }
            }
            catch (Exception ex)
            {
                Console.WriteLine($"{taskName} 下载失败: {ex.Message}, Retrying in {retryDelayMs}ms...");
            }
            retries++;
            await Task.Delay(retryDelayMs);
        }
        Console.WriteLine($"{taskName} 下载失败，尝试次数超过限制.");
    }

    static bool VerifyFileSha1(string filePath, string expectedSha1)
    {
        if (string.IsNullOrEmpty(expectedSha1)) return true; //No SHA1 to verify
        using (var sha1 = SHA1.Create())
        {
            using (var stream = File.OpenRead(filePath))
            {
                var hashBytes = sha1.ComputeHash(stream);
                var actualSha1 = BitConverter.ToString(hashBytes).Replace("-", "").ToLowerInvariant();
                return actualSha1 == expectedSha1;
            }
        }
    }

    static async Task DownloadAndExtractLibraryWithRateLimit(JsonElement downloadInfo, string filePath, string taskName, SemaphoreSlim semaphore,string version)
    {
        await semaphore.WaitAsync();
        try
        {
            await DownloadAndVerify(downloadInfo, filePath, taskName);
            ExtractWindowsNatives(filePath,version);
        }
        catch (Exception ex)
        {
            libraryExtractionErrors.Enqueue(ex);
            Console.WriteLine($"Error processing library {taskName}: {ex.Message}");
        }
        finally
        {
            semaphore.Release();
        }
    }

    static void ExtractWindowsNatives(string libPath,string versionD)
    {
        try
        {
            string nativesDir = Path.Combine(basePath, "versions", versionD, $"{versionD}-natives");
            Directory.CreateDirectory(nativesDir);

            using (ZipArchive archive = ZipFile.OpenRead(libPath))
            {
                foreach (ZipArchiveEntry entry in archive.Entries)
                {
                    if (entry.FullName.EndsWith(".dll", StringComparison.OrdinalIgnoreCase))
                    {
                        string destinationPath = Path.Combine(nativesDir, entry.Name);
                        entry.ExtractToFile(destinationPath, true);
                    }
                }
            }
        }
        catch (Exception ex)
        {
            libraryExtractionErrors.Enqueue(ex);
            Console.WriteLine($"Error extracting natives from {libPath}: {ex.Message}");
        }
    }
}

public class Counter
{
    private int _value = 0;
    public int Value => _value;
    public void Increment() => Interlocked.Increment(ref _value);
}