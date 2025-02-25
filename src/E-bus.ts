import type { EventType, EventPayloadMap, EventHandler } from './event';

class EventBus {
  // 存储结构：事件类型 => 原始处理器 => 包装器
  private handlerMap = new Map<
    EventType,
    Map<EventHandler<any>, Set<Function>>
  >();

  // 发布事件
  emit<T extends EventType>(event: T, payload: EventPayloadMap[T]): void {
    const handlers = this.handlerMap.get(event);
    if (!handlers) return;

    handlers.forEach((wrappers) => {
      wrappers.forEach(wrapper => wrapper(payload));
    });
  }

  // 订阅事件
  on<T extends EventType>(event: T, handler: EventHandler<T>) {
    const wrapper = (payload: EventPayloadMap[T]) => handler(payload);

    if (!this.handlerMap.has(event)) {
      this.handlerMap.set(event, new Map());
    }

    const eventHandlers = this.handlerMap.get(event)!;
    if (!eventHandlers.has(handler)) {
      eventHandlers.set(handler, new Set());
    }

    eventHandlers.get(handler)!.add(wrapper);

    return {
      unsubscribe: () => this.off(event, handler)
    };
  }

  // 取消订阅
  off<T extends EventType>(event: T, handler: EventHandler<T>) {
    const handlers = this.handlerMap.get(event);
    if (!handlers) return;

    const wrappers = handlers.get(handler);
    if (!wrappers) return;

    wrappers.clear();
    handlers.delete(handler);
  }
}

export const eventBus = new EventBus();
