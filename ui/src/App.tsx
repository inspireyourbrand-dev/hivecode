import React, { useEffect } from "react";
import { useTheme } from "@/hooks/useTheme";
import { useAppStore } from "@/stores/appStore";
import { Header } from "@/components/Header";
import { Sidebar } from "@/components/Sidebar";
import { ChatPanel } from "@/components/ChatPanel";
import { SettingsPanel } from "@/components/SettingsPanel";
import { NotificationProvider } from "@/components/NotificationToast";
import { KeyboardShortcuts } from "@/components/KeyboardShortcuts";
import { listProviders, listTools } from "@/lib/tauri";
import "@/styles/globals.css";

function AppContent() {
  useTheme();
  const setProviders = useAppStore((state) => state.setProviders);
  const setTools = useAppStore((state) => state.setTools);

  useEffect(() => {
    const loadAppData = async () => {
      try {
        const providers = await listProviders();
        setProviders(providers);

        const tools = await listTools();
        setTools(tools);
      } catch (error) {
        console.error("Failed to load app data:", error);
      }
    };

    loadAppData();
  }, [setProviders, setTools]);

  return (
    <div className="flex h-screen w-screen overflow-hidden bg-hive-bg-light dark:bg-hive-bg">
      {/* Sidebar */}
      <Sidebar />

      {/* Main Content */}
      <div className="flex-1 flex flex-col overflow-hidden">
        {/* Header */}
        <Header />

        {/* Chat Area */}
        <div className="flex-1 overflow-hidden">
          <ChatPanel />
        </div>
      </div>

      {/* Settings Modal */}
      <SettingsPanel />

      {/* Keyboard Shortcuts */}
      <KeyboardShortcuts />
    </div>
  );
}

function App() {
  return (
    <NotificationProvider>
      <AppContent />
    </NotificationProvider>
  );
}

export default App;
