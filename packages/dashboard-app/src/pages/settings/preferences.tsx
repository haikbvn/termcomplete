import { UserPrefView } from "@/components/preference/list";
import preferences from "@/data/preferences";
export default function Page() { return <UserPrefView array={preferences} />; }
