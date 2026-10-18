import { Routes, Route, NavLink } from "react-router-dom";
import { useStore } from "zustand";
import { StoreContext } from "./context/zustand";
import { createStore } from "./lib/store";
import Autocomplete from "./pages/terminal/autocomplete";
import Keybindings from "./pages/settings/keybindings";
import Preferences from "./pages/settings/preferences";
import Setup from "./pages/onboarding";
const store = createStore();
export default function App() {
  const loading = useStore(store, (s) => s.isLoading());
  if (loading) return <p className="p-8">Loading Termcomplete settings…</p>;
  return <StoreContext.Provider value={store}>
    <div className="flex h-screen bg-white text-zinc-800 dark:bg-zinc-950 dark:text-zinc-200">
      <aside className="w-56 shrink-0 border-r border-zinc-200 dark:border-zinc-800 p-5 flex flex-col gap-6">
        <div className="font-semibold text-lg">〉_ Termcomplete</div>
        <nav className="flex flex-col gap-1">{[["/", "Setup"], ["/autocomplete", "Autocomplete"], ["/keybindings", "Keybindings"], ["/preferences", "Preferences"]].map(([path, name]) =>
          <NavLink key={path} to={path} end className={({isActive}) => `rounded-lg px-3 py-2 text-sm ${isActive ? "bg-dusk-600 text-white" : "hover:bg-zinc-100 dark:hover:bg-zinc-900"}`}>{name}</NavLink>)}</nav>
        <p className="mt-auto text-xs text-zinc-500">Local autocomplete · v0.1.0</p>
      </aside>
      <main className="min-w-0 flex-1 overflow-auto p-8"><Routes>
        <Route path="/" element={<Setup/>}/><Route path="/autocomplete" element={<Autocomplete/>}/>
        <Route path="/keybindings" element={<Keybindings/>}/><Route path="/preferences" element={<Preferences/>}/>
        <Route path="*" element={<Setup/>}/>
      </Routes></main>
    </div>
  </StoreContext.Provider>;
}
