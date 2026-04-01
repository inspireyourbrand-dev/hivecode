import React, { useEffect } from "react";
import { useNotificationStore, Notification } from "@/stores/notificationStore";
import { X, CheckCircle, AlertCircle, AlertTriangle, Info } from "lucide-react";

const getNotificationColors = (
  type: Notification["type"]
): {
  bg: string;
  border: string;
  icon: React.ReactNode;
} => {
  switch (type) {
    case "success":
      return {
        bg: "dark:bg-green-950 bg-green-50",
        border: "border-hive-green",
        icon: <CheckCircle className="w-5 h-5 text-hive-green" />,
      };
    case "error":
      return {
        bg: "dark:bg-red-950 bg-red-50",
        border: "border-red-500",
        icon: <AlertCircle className="w-5 h-5 text-red-500" />,
      };
    case "warning":
      return {
        bg: "dark:bg-yellow-950 bg-yellow-50",
        border: "border-hive-yellow",
        icon: <AlertTriangle className="w-5 h-5 text-hive-yellow" />,
      };
    case "info":
      return {
        bg: "dark:bg-blue-950 bg-blue-50",
        border: "border-hive-cyan",
        icon: <Info className="w-5 h-5 text-hive-cyan" />,
      };
  }
};

const ToastItem: React.FC<{ notification: Notification }> = ({
  notification,
}) => {
  const removeNotification = useNotificationStore(
    (state) => state.removeNotification
  );
  const { bg, border, icon } = getNotificationColors(notification.type);

  return (
    <div
      className={`flex gap-3 items-start p-4 rounded-lg border ${bg} ${border} animate-slide-in max-w-sm`}
      role="alert"
    >
      <div className="flex-shrink-0">{icon}</div>

      <div className="flex-1 min-w-0">
        <h3 className="font-semibold text-sm dark:text-white text-slate-900">
          {notification.title}
        </h3>
        {notification.message && (
          <p className="text-xs mt-1 dark:text-slate-400 text-slate-600">
            {notification.message}
          </p>
        )}
      </div>

      {notification.dismissible && (
        <button
          onClick={() => removeNotification(notification.id)}
          className="flex-shrink-0 p-1 hover:bg-white/10 dark:hover:bg-black/20 rounded transition-colors"
          title="Dismiss"
        >
          <X className="w-4 h-4 dark:text-slate-400 text-slate-600" />
        </button>
      )}
    </div>
  );
};

export const NotificationProvider: React.FC<{
  children: React.ReactNode;
}> = ({ children }) => {
  const notifications = useNotificationStore((state) => state.notifications);

  return (
    <>
      {children}

      {/* Toast Container */}
      <div className="fixed top-4 right-4 z-50 flex flex-col gap-3 pointer-events-auto">
        {notifications.slice(-5).map((notification) => (
          <ToastItem
            key={notification.id}
            notification={notification}
          />
        ))}
      </div>
    </>
  );
};

export const useNotification = () => {
  const addNotification = useNotificationStore(
    (state) => state.addNotification
  );

  return {
    success: (title: string, message?: string, duration?: number) =>
      addNotification({
        type: "success",
        title,
        message,
        duration,
        dismissible: true,
      }),
    error: (title: string, message?: string, duration?: number) =>
      addNotification({
        type: "error",
        title,
        message,
        duration,
        dismissible: true,
      }),
    warning: (title: string, message?: string, duration?: number) =>
      addNotification({
        type: "warning",
        title,
        message,
        duration,
        dismissible: true,
      }),
    info: (title: string, message?: string, duration?: number) =>
      addNotification({
        type: "info",
        title,
        message,
        duration,
        dismissible: true,
      }),
  };
};
