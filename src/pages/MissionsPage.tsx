import { useEffect, useState } from "react";
import Header from "@/components/Header";
import MissionCard from "@/components/MissionCard";
import EmptyState from "@/components/EmptyState";
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
      {loading ? (
        <div className="loading">Loading missions...</div>
      ) : missions.length === 0 ? (
        <EmptyState title="No missions" description="No missions found." />
      ) : (
        <div className="cards-grid">
          {missions.map((m: any) => (
            <MissionCard key={m.id} mission={m} />
          ))}
        </div>
      )}
    </>
  );
}
