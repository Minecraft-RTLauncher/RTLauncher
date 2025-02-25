import { ref } from "vue";

type AnimationType = string;

class AnimationManager {
  private static _instance: AnimationManager;
  private _availableAnimations = new Set<AnimationType>();
  private _currentAnimation = ref('');
  private _styleObserver: MutationObserver | null = null;
  private _cssPrefix = 'animate-';

  private constructor() {
    this._initAnimationDetection();
  }

  public static get instance(): AnimationManager {
    if (!this._instance) {
      this._instance = new AnimationManager();
    }
    return this._instance;
  }

  // 获取所有可用动画列表
  public get animations(): AnimationType[] {
    return Array.from(this._availableAnimations);
  }

  // 设置当前动画
  public setAnimation(name: AnimationType): void {
    if (this._validateAnimation(name)) {
      this._currentAnimation.value = name;
    } else {
      console.warn(`Animation "${name}" is not registered`);
    }
  }

  // 获取当前动画类名
  public get currentAnimationClass(): string {
    return `${this._cssPrefix}${this._currentAnimation.value}`;
  }

  // 初始化动画检测系统
  private _initAnimationDetection(): void {
    this._scanAllStylesheets();
    this._setupStyleObserver();
  }

  // 扫描现有样式表
  private _scanAllStylesheets(): void {
    Array.from(document.styleSheets).forEach(sheet => {
      this._processStyleSheet(sheet);
    });
  }

  // 设置样式变化监听
  private _setupStyleObserver(): void {
    this._styleObserver = new MutationObserver(mutations => {
      mutations.forEach(mutation => {
        if (mutation.type === 'childList') {
          Array.from(mutation.addedNodes).forEach(node => {
            if (node instanceof HTMLLinkElement && node.rel === 'stylesheet') {
              node.addEventListener('load', () => {
                this._processStyleSheet(node.sheet!);
              });
            }
          });
        }
      });
    });

    this._styleObserver.observe(document.head, {
      childList: true,
      subtree: true
    });
  }

  // 处理单个样式表
  private _processStyleSheet(sheet: CSSStyleSheet): void {
    try {
      Array.from(sheet.cssRules).forEach(rule => {
        if (rule instanceof CSSStyleRule) {
          this._extractAnimationFromRule(rule);
        }
      });
    } catch (e) {
      console.warn('Cannot access stylesheet:', e);
    }
  }

  // 从CSS规则提取动画
  private _extractAnimationFromRule(rule: CSSStyleRule): void {
    const animationRegex = new RegExp(`^\.${this._cssPrefix}([a-zA-Z0-9_-]+)$`);
    const match = rule.selectorText.match(animationRegex);
    
    if (match && match[1]) {
      this._availableAnimations.add(match[1]);
    }
  }

  // 验证动画是否存在
  private _validateAnimation(name: string): boolean {
    return this._availableAnimations.has(name);
  }
}

export const animationManager = AnimationManager.instance;