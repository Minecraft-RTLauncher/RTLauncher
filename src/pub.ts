import { eventBus } from './E-bus';
import type { EventType, EventPayloadMap } from './event';

// 🌈 通用事件发布器（自动类型推断）
export const emitEvent = <T extends EventType>(
  event: T,
  payload: EventPayloadMap[T]
): void => {
  eventBus.emit(event, payload);
};

// 🚀 快捷方法示例
export const publishMessage = (content: string) => 
  emitEvent('message-created', {
    id: crypto.randomUUID(),
    content,
    timestamp: Date.now()
  });