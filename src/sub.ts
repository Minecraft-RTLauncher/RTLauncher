import { eventBus } from './E-bus';
import type { EventType, EventHandler } from './event';

export class EventSubscriber {
  private subscriptions: ReturnType<typeof eventBus.on>[] = [];

  // 🛡️ 类型安全订阅
  subscribe<T extends EventType>(event: T, handler: EventHandler<T>) {
    const sub = eventBus.on(event, handler);
    this.subscriptions.push(sub);
    return this;
  }

  // 🧼 自动清理订阅
  unsubscribeAll() {
    this.subscriptions.forEach(sub => sub.unsubscribe());
    this.subscriptions = [];
  }
}