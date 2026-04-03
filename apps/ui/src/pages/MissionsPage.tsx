import { useEffect, useState } from "react";
import Header from "@/components/Header";
import MissionCard from "@/components/MissionCard";
import { DataState } from "@/components/ui";
import { api } from "@/lib/api";

export default function MissionsPage() {
  const [missions, setMissions] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    api.getMissions().then((data) => {
      setMissions(data.missions || []);
      setLoading(false);
    }).catch(() => setLoading(false));
  }, []);

  return (
    <>
      <Header title="Missions" />
      <DataState loading={loading} empty={missions.length === 0} emptyTitle="No missions" emptyDescription="No missions found." loadingText="Loading missions...">
        <div className="cards-grid">
          {missions.map((m: any) => (
            <MissionCard key={m.id} mission={m} />
          ))}
        </div>
      </DataState>
    </>
  );
}
