import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import {
  X,
  Plus,
  Trash2,
  CheckCircle,
  AlertCircle,
  Copy,
  Eye,
  EyeOff,
} from "lucide-react";
import { useAppStore } from "@/stores/appStore";

interface AuthProfile {
  id: string;
  provider: string;
  display_name?: string;
  email?: string;
  is_default: boolean;
  created_at: string;
  last_used?: string;
  auth_type: string;
  expires_at?: number;
}

interface AuthTestResult {
  success: boolean;
  message: string;
  provider: string;
  auth_type: string;
}

export const AuthPanel: React.FC = () => {
  const authPanelOpen = useAppStore((state) => state.authPanelOpen);
  const toggleAuthPanel = useAppStore((state) => state.toggleAuthPanel);
  const theme = useAppStore((state) => state.theme);

  const [activeTab, setActiveTab] = useState<
    "openai" | "anthropic" | "ollama" | "chatgpt"
  >("openai");
  const [profiles, setProfiles] = useState<AuthProfile[]>([]);
  const [loading, setLoading] = useState(false);
  const [testingId, setTestingId] = useState<string | null>(null);
  const [testResults, setTestResults] = useState<Record<string, AuthTestResult>>(
    {}
  );

  // API Key input state
  const [apiKeyInput, setApiKeyInput] = useState("");
  const [displayNameInput, setDisplayNameInput] = useState("");
  const [showApiKey, setShowApiKey] = useState(false);

  // ChatGPT session state
  const [sessionTokenInput, setSessionTokenInput] = useState("");
  const [showSessionToken, setShowSessionToken] = useState(false);
  const [chatGptInstructions, setChatGptInstructions] = useState("");
  const [showInstructions, setShowInstructions] = useState(false);

  useEffect(() => {
    if (authPanelOpen) {
      loadProfiles();
    }
  }, [authPanelOpen]);

  const loadProfiles = async () => {
    setLoading(true);
    try {
      const result = await invoke<AuthProfile[]>("list_auth_profiles");
      setProfiles(result);
    } catch (error) {
      console.error("Failed to load auth profiles:", error);
    } finally {
      setLoading(false);
    }
  };

  const handleAddApiKey = async () => {
    if (!apiKeyInput.trim()) {
      alert("Please enter an API key");
      return;
    }

    try {
      await invoke("add_api_key_profile", {
        provider: activeTab === "chatgpt" ? "openai" : activeTab,
        api_key: apiKeyInput,
        display_name: displayNameInput || undefined,
      });

      setApiKeyInput("");
      setDisplayNameInput("");
      await loadProfiles();
    } catch (error) {
      console.error("Failed to add API key:", error);
      alert(`Error: ${error}`);
    }
  };

  const handleRemoveProfile = async (id: string) => {
    if (!confirm("Are you sure you want to remove this profile?")) {
      return;
    }

    try {
      await invoke("remove_auth_profile", { id });
      await loadProfiles();
    } catch (error) {
      console.error("Failed to remove profile:", error);
      alert(`Error: ${error}`);
    }
  };

  const handleSetDefault = async (id: string) => {
    try {
      await invoke("set_default_profile", { id });
      await loadProfiles();
    } catch (error) {
      console.error("Failed to set default profile:", error);
      alert(`Error: ${error}`);
    }
  };

  const handleTestProfile = async (id: string) => {
    setTestingId(id);
    try {
      const result = await invoke<AuthTestResult>("test_auth_profile", { id });
      setTestResults((prev) => ({ ...prev, [id]: result }));
    } catch (error) {
      console.error("Failed to test profile:", error);
      setTestResults((prev) => ({
        ...prev,
        [id]: {
          success: false,
          message: `Error: ${error}`,
          provider: "",
          auth_type: "",
        },
      }));
    } finally {
      setTestingId(null);
    }
  };

  const handleAddChatGptSession = async () => {
    if (!sessionTokenInput.trim()) {
      alert("Please enter a session token");
      return;
    }

    try {
      await invoke("add_chatgpt_session", {
        session_token: sessionTokenInput,
        display_name: displayNameInput || "ChatGPT Subscription",
      });

      setSessionTokenInput("");
      setDisplayNameInput("");
      await loadProfiles();
    } catch (error) {
      console.error("Failed to add ChatGPT session:", error);
      alert(`Error: ${error}`);
    }
  };

  const handleGetChatGptInstructions = async () => {
    try {
      const instructions = await invoke<string>(
        "get_chatgpt_login_instructions"
      );
      setChatGptInstructions(instructions);
    } catch (error) {
      console.error("Failed to get instructions:", error);
    }
  };

  const getProviderColor = (provider: string) => {
    switch (provider) {
      case "openai":
        return "text-green-500 bg-green-500/10";
      case "anthropic":
        return "text-amber-500 bg-amber-500/10";
      case "chatgpt":
        return "text-cyan-500 bg-cyan-500/10";
      case "ollama":
        return "text-purple-500 bg-purple-500/10";
      default:
        return "text-gray-500 bg-gray-500/10";
    }
  };

  const filteredProfiles = profiles.filter((p) => {
    if (activeTab === "chatgpt") return p.provider === "chatgpt";
    return p.provider === activeTab;
  });

  if (!authPanelOpen) return null;

  return (
    <>
      {/* Backdrop */}
      <div
        className="fixed inset-0 bg-black/50 z-40"
        onClick={toggleAuthPanel}
      />

      {/* Modal */}
      <div className="fixed inset-4 md:inset-auto md:left-1/2 md:top-1/2 md:w-3xl md:max-h-[90vh] md:transform md:-translate-x-1/2 md:-translate-y-1/2 bg-white dark:bg-hive-surface rounded-lg shadow-2xl z-50 flex flex-col overflow-hidden">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-hive-border-light dark:border-hive-border">
          <h2 className="text-xl font-bold text-slate-900 dark:text-white">
            Authentication
          </h2>
          <button
            onClick={toggleAuthPanel}
            className="p-2 hover:bg-hive-bg-light dark:hover:bg-hive-surface rounded transition-colors"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Tabs */}
        <div className="flex border-b border-hive-border-light dark:border-hive-border px-6">
          {(["openai", "anthropic", "ollama", "chatgpt"] as const).map(
            (tab) => (
              <button
                key={tab}
                onClick={() => setActiveTab(tab)}
                className={`px-4 py-3 text-sm font-medium border-b-2 transition-colors capitalize ${
                  activeTab === tab
                    ? "border-hive-cyan text-hive-cyan"
                    : "border-transparent text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-white"
                }`}
              >
                {tab}
              </button>
            )
          )}
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto px-6 py-6">
          {activeTab === "chatgpt" ? (
            <ChatGptTabContent
              sessionTokenInput={sessionTokenInput}
              setSessionTokenInput={setSessionTokenInput}
              displayNameInput={displayNameInput}
              setDisplayNameInput={setDisplayNameInput}
              showSessionToken={showSessionToken}
              setShowSessionToken={setShowSessionToken}
              chatGptInstructions={chatGptInstructions}
              showInstructions={showInstructions}
              setShowInstructions={setShowInstructions}
              handleGetChatGptInstructions={handleGetChatGptInstructions}
              handleAddChatGptSession={handleAddChatGptSession}
            />
          ) : (
            <ApiKeyTabContent
              apiKeyInput={apiKeyInput}
              setApiKeyInput={setApiKeyInput}
              displayNameInput={displayNameInput}
              setDisplayNameInput={setDisplayNameInput}
              showApiKey={showApiKey}
              setShowApiKey={setShowApiKey}
              handleAddApiKey={handleAddApiKey}
            />
          )}

          {/* Profiles List */}
          <div className="mt-8">
            <h3 className="text-lg font-semibold text-slate-900 dark:text-white mb-4">
              Saved Profiles
            </h3>

            {loading ? (
              <div className="text-center py-8">
                <p className="text-slate-600 dark:text-slate-400">
                  Loading profiles...
                </p>
              </div>
            ) : filteredProfiles.length === 0 ? (
              <div className="text-center py-8">
                <p className="text-slate-600 dark:text-slate-400">
                  No profiles yet. Add one above to get started.
                </p>
              </div>
            ) : (
              <div className="space-y-3">
                {filteredProfiles.map((profile) => (
                  <ProfileCard
                    key={profile.id}
                    profile={profile}
                    isDefault={profile.is_default}
                    testResult={testResults[profile.id]}
                    isTesting={testingId === profile.id}
                    onSetDefault={() => handleSetDefault(profile.id)}
                    onTest={() => handleTestProfile(profile.id)}
                    onRemove={() => handleRemoveProfile(profile.id)}
                    getProviderColor={getProviderColor}
                  />
                ))}
              </div>
            )}
          </div>
        </div>
      </div>
    </>
  );
};

interface ApiKeyTabContentProps {
  apiKeyInput: string;
  setApiKeyInput: (value: string) => void;
  displayNameInput: string;
  setDisplayNameInput: (value: string) => void;
  showApiKey: boolean;
  setShowApiKey: (value: boolean) => void;
  handleAddApiKey: () => void;
}

const ApiKeyTabContent: React.FC<ApiKeyTabContentProps> = ({
  apiKeyInput,
  setApiKeyInput,
  displayNameInput,
  setDisplayNameInput,
  showApiKey,
  setShowApiKey,
  handleAddApiKey,
}) => (
  <div className="space-y-4">
    <div>
      <label className="block text-sm font-medium text-slate-900 dark:text-white mb-2">
        Display Name (optional)
      </label>
      <input
        type="text"
        placeholder="e.g., My API Key"
        value={displayNameInput}
        onChange={(e) => setDisplayNameInput(e.target.value)}
        className="w-full px-4 py-2 rounded-lg border border-hive-border-light dark:border-hive-border bg-white dark:bg-hive-bg text-slate-900 dark:text-white placeholder-slate-500 dark:placeholder-slate-400 focus:outline-none focus:border-hive-cyan focus:ring-1 focus:ring-hive-cyan transition-colors"
      />
    </div>

    <div>
      <label className="block text-sm font-medium text-slate-900 dark:text-white mb-2">
        API Key
      </label>
      <div className="relative">
        <input
          type={showApiKey ? "text" : "password"}
          placeholder="sk-..."
          value={apiKeyInput}
          onChange={(e) => setApiKeyInput(e.target.value)}
          className="w-full px-4 py-2 pr-10 rounded-lg border border-hive-border-light dark:border-hive-border bg-white dark:bg-hive-bg text-slate-900 dark:text-white placeholder-slate-500 dark:placeholder-slate-400 focus:outline-none focus:border-hive-cyan focus:ring-1 focus:ring-hive-cyan transition-colors"
        />
        <button
          onClick={() => setShowApiKey(!showApiKey)}
          className="absolute right-3 top-1/2 transform -translate-y-1/2 text-slate-500 hover:text-slate-700 dark:text-slate-400 dark:hover:text-slate-200"
        >
          {showApiKey ? <EyeOff size={18} /> : <Eye size={18} />}
        </button>
      </div>
    </div>

    <button
      onClick={handleAddApiKey}
      className="w-full px-4 py-2 bg-hive-cyan hover:bg-hive-cyan/90 text-white rounded-lg font-medium transition-colors flex items-center justify-center gap-2"
    >
      <Plus size={18} />
      Add API Key
    </button>
  </div>
);

interface ChatGptTabContentProps {
  sessionTokenInput: string;
  setSessionTokenInput: (value: string) => void;
  displayNameInput: string;
  setDisplayNameInput: (value: string) => void;
  showSessionToken: boolean;
  setShowSessionToken: (value: boolean) => void;
  chatGptInstructions: string;
  showInstructions: boolean;
  setShowInstructions: (value: boolean) => void;
  handleGetChatGptInstructions: () => void;
  handleAddChatGptSession: () => void;
}

const ChatGptTabContent: React.FC<ChatGptTabContentProps> = ({
  sessionTokenInput,
  setSessionTokenInput,
  displayNameInput,
  setDisplayNameInput,
  showSessionToken,
  setShowSessionToken,
  chatGptInstructions,
  showInstructions,
  setShowInstructions,
  handleGetChatGptInstructions,
  handleAddChatGptSession,
}) => (
  <div className="space-y-6">
    {/* Info Box */}
    <div className="p-4 bg-cyan-50 dark:bg-cyan-500/10 border border-cyan-200 dark:border-cyan-500/30 rounded-lg">
      <p className="text-sm text-cyan-900 dark:text-cyan-200">
        <strong>ChatGPT Subscription:</strong> If you have ChatGPT Plus, Pro, or
        Team subscription, you can use your subscription instead of paying for
        API access separately. Your session token will be securely exchanged for
        an access token.
      </p>
    </div>

    {/* Instructions Button */}
    <button
      onClick={() => {
        setShowInstructions(!showInstructions);
        if (!showInstructions && !chatGptInstructions) {
          handleGetChatGptInstructions();
        }
      }}
      className="w-full px-4 py-2 border border-hive-border-light dark:border-hive-border hover:bg-hive-bg-light dark:hover:bg-hive-surface rounded-lg font-medium transition-colors text-slate-900 dark:text-white"
    >
      {showInstructions ? "Hide" : "Show"} Instructions
    </button>

    {/* Instructions */}
    {showInstructions && chatGptInstructions && (
      <div className="p-4 bg-slate-50 dark:bg-hive-bg rounded-lg border border-hive-border-light dark:border-hive-border">
        <ol className="text-sm text-slate-700 dark:text-slate-300 space-y-2 list-decimal list-inside">
          {chatGptInstructions.split("\n").map((line, idx) => (
            <li key={idx} className="break-words">
              {line}
            </li>
          ))}
        </ol>
      </div>
    )}

    {/* Display Name Input */}
    <div>
      <label className="block text-sm font-medium text-slate-900 dark:text-white mb-2">
        Display Name (optional)
      </label>
      <input
        type="text"
        placeholder="e.g., My ChatGPT Subscription"
        value={displayNameInput}
        onChange={(e) => setDisplayNameInput(e.target.value)}
        className="w-full px-4 py-2 rounded-lg border border-hive-border-light dark:border-hive-border bg-white dark:bg-hive-bg text-slate-900 dark:text-white placeholder-slate-500 dark:placeholder-slate-400 focus:outline-none focus:border-hive-cyan focus:ring-1 focus:ring-hive-cyan transition-colors"
      />
    </div>

    {/* Session Token Input */}
    <div>
      <label className="block text-sm font-medium text-slate-900 dark:text-white mb-2">
        Session Token
      </label>
      <div className="relative">
        <textarea
          placeholder="Paste your __Secure-next-auth.session-token here..."
          value={sessionTokenInput}
          onChange={(e) => setSessionTokenInput(e.target.value)}
          className="w-full px-4 py-2 rounded-lg border border-hive-border-light dark:border-hive-border bg-white dark:bg-hive-bg text-slate-900 dark:text-white placeholder-slate-500 dark:placeholder-slate-400 focus:outline-none focus:border-hive-cyan focus:ring-1 focus:ring-hive-cyan transition-colors font-mono text-xs"
          rows={4}
        />
        {sessionTokenInput && (
          <button
            onClick={() => {
              navigator.clipboard.writeText(sessionTokenInput);
            }}
            className="absolute right-3 top-3 text-slate-500 hover:text-slate-700 dark:text-slate-400 dark:hover:text-slate-200"
            title="Token is secure and only stored locally"
          >
            <Copy size={16} />
          </button>
        )}
      </div>
      <p className="text-xs text-slate-500 dark:text-slate-400 mt-2">
        Your token is stored securely and never shared.
      </p>
    </div>

    <button
      onClick={handleAddChatGptSession}
      className="w-full px-4 py-2 bg-hive-cyan hover:bg-hive-cyan/90 text-white rounded-lg font-medium transition-colors flex items-center justify-center gap-2"
    >
      <Plus size={18} />
      Add ChatGPT Session
    </button>
  </div>
);

interface ProfileCardProps {
  profile: AuthProfile;
  isDefault: boolean;
  testResult?: AuthTestResult;
  isTesting: boolean;
  onSetDefault: () => void;
  onTest: () => void;
  onRemove: () => void;
  getProviderColor: (provider: string) => string;
}

const ProfileCard: React.FC<ProfileCardProps> = ({
  profile,
  isDefault,
  testResult,
  isTesting,
  onSetDefault,
  onTest,
  onRemove,
  getProviderColor,
}) => (
  <div className="p-4 border border-hive-border-light dark:border-hive-border rounded-lg bg-slate-50 dark:bg-hive-bg hover:shadow-md transition-shadow">
    <div className="flex items-start justify-between mb-3">
      <div className="flex-1">
        <div className="flex items-center gap-2 mb-1">
          <span
            className={`px-2 py-1 rounded text-xs font-semibold uppercase ${getProviderColor(
              profile.provider
            )}`}
          >
            {profile.provider}
          </span>
          {isDefault && (
            <span className="px-2 py-1 rounded text-xs font-semibold uppercase bg-green-500/10 text-green-600 dark:text-green-400">
              Default
            </span>
          )}
        </div>
        <p className="font-medium text-slate-900 dark:text-white">
          {profile.display_name || `${profile.provider} Profile`}
        </p>
        {profile.email && (
          <p className="text-sm text-slate-600 dark:text-slate-400">
            {profile.email}
          </p>
        )}
      </div>

      <button
        onClick={onRemove}
        className="p-2 text-red-500 hover:bg-red-50 dark:hover:bg-red-500/10 rounded transition-colors"
        title="Remove profile"
      >
        <Trash2 size={18} />
      </button>
    </div>

    {/* Test Result */}
    {testResult && (
      <div
        className={`mb-3 p-2 rounded text-sm flex items-start gap-2 ${
          testResult.success
            ? "bg-green-50 dark:bg-green-500/10 text-green-700 dark:text-green-300"
            : "bg-red-50 dark:bg-red-500/10 text-red-700 dark:text-red-300"
        }`}
      >
        {testResult.success ? (
          <CheckCircle size={16} className="flex-shrink-0 mt-0.5" />
        ) : (
          <AlertCircle size={16} className="flex-shrink-0 mt-0.5" />
        )}
        <span>{testResult.message}</span>
      </div>
    )}

    {/* Metadata */}
    <div className="text-xs text-slate-500 dark:text-slate-400 mb-3 space-y-1">
      <p>
        Created: {new Date(profile.created_at).toLocaleDateString()} at{" "}
        {new Date(profile.created_at).toLocaleTimeString()}
      </p>
      {profile.last_used && (
        <p>
          Last used: {new Date(profile.last_used).toLocaleDateString()} at{" "}
          {new Date(profile.last_used).toLocaleTimeString()}
        </p>
      )}
      {profile.expires_at && (
        <p>
          Expires: {new Date(profile.expires_at * 1000).toLocaleDateString()}
        </p>
      )}
    </div>

    {/* Actions */}
    <div className="flex gap-2">
      {!isDefault && (
        <button
          onClick={onSetDefault}
          className="flex-1 px-3 py-2 text-sm font-medium border border-hive-border-light dark:border-hive-border hover:bg-hive-bg-light dark:hover:bg-hive-surface rounded transition-colors text-slate-900 dark:text-white"
        >
          Set as Default
        </button>
      )}

      <button
        onClick={onTest}
        disabled={isTesting}
        className="flex-1 px-3 py-2 text-sm font-medium border border-hive-cyan text-hive-cyan hover:bg-hive-cyan/10 rounded transition-colors disabled:opacity-50"
      >
        {isTesting ? "Testing..." : "Test"}
      </button>
    </div>
  </div>
);
