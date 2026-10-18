import { useEffect, useState } from "react";
import { Install, Internal, Fig } from "@aws/amazon-q-developer-cli-api-bindings";
import { Button } from "@/components/ui/button";
export default function Setup() {
  const [status, setStatus] = useState<Record<string, boolean>>({});
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const refresh = async () => {
    const keys = ["dotfiles", "accessibility", "inputMethod"] as const;
    const values = await Promise.all(keys.map(async key => [key, await Install.isInstalled(key)] as const));
    setStatus(Object.fromEntries(values));
  };
  useEffect(() => { refresh().catch(e => setError(String(e))); const timer = setInterval(() => refresh().catch(() => {}), 3000); return () => clearInterval(timer); }, []);
  async function install(key: "dotfiles" | "accessibility" | "inputMethod") {
    setError(""); setBusy(true);
    try { await Install.install(key); await refresh(); }
    catch (e) { setError(String(e)); } finally { setBusy(false); }
  }
  async function finish() {
    try { await Internal.sendOnboardingRequest({action: Fig.OnboardingAction.FINISH_ONBOARDING}); }
    catch (e) { setError(String(e)); }
  }
  return <div className="max-w-2xl flex flex-col gap-6">
    <div><h1 className="text-2xl font-semibold">Set up Termcomplete</h1><p className="mt-2 text-sm text-zinc-500">Command suggestions for Bash and Zsh in Terminal and iTerm2.</p></div>
    {([
      ["dotfiles", "Shell integration", "Install hooks in your Bash and Zsh startup files. Existing files are backed up before changes."],
      ["accessibility", "Accessibility", "Allow Termcomplete in System Settings → Privacy & Security → Accessibility."],
      ["inputMethod", "Cursor positioning helper", "Register the bundled input method so suggestions can follow the terminal cursor."],
    ] as const).map(([key,title,description]) => <section key={key} className="rounded-lg border border-zinc-200 dark:border-zinc-700 p-5 flex items-start justify-between gap-4">
      <div><h2 className="font-medium">{title}</h2><p className="mt-1 text-sm text-zinc-500">{description}</p><p className="mt-2 text-xs">{status[key] ? "Ready" : "Setup required"}</p></div>
      <Button disabled={busy || status[key]} onClick={() => install(key)}>{key === "accessibility" ? "Open settings" : "Install"}</Button>
    </section>)}
    {error && <p role="alert" className="text-sm text-red-600">{error}</p>}
    <p className="text-sm text-zinc-500">After setup, open a new terminal window and type <code>git </code>. Bundled definitions work offline; some CLI generators query their own services.</p>
    <Button disabled={!Object.values(status).length || !Object.values(status).every(Boolean)} onClick={finish}>Finish setup</Button>
  </div>;
}
