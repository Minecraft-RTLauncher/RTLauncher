export const SystemEvents = {
    MESSAGE_CREATED: 'message-created',
    USER_LOGIN: 'user-login',
    // 🆕 在此处添加新事件类型
  } as const;
  
  // 自动生成所有事件类型联合
  export type EventType = typeof SystemEvents[keyof typeof SystemEvents];
  
  // 核心类型映射（在此处添加新事件payload）
  export type EventPayloadMap = {
    [SystemEvents.MESSAGE_CREATED]: {
      id: string;
      content: string;
      timestamp: number;
    };
    [SystemEvents.USER_LOGIN]: {
      userId: string;
      authType: 'password' | 'oauth';
    };
    // 🆕 新事件在此添加payload类型
  };
  
  // 类型安全的事件处理器类型
  export type EventHandler<T extends EventType> = (payload: EventPayloadMap[T]) => void;