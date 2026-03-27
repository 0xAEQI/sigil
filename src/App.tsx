import { Routes, Route, Navigate } from "react-router-dom";
import { useAuthStore } from "@/store/auth";
import AppLayout from "@/components/AppLayout";
import LoginPage from "@/pages/LoginPage";
import DashboardPage from "@/pages/DashboardPage";
import ProjectsPage from "@/pages/ProjectsPage";
import AgentsPage from "@/pages/AgentsPage";
import TasksPage from "@/pages/TasksPage";
import MissionsPage from "@/pages/MissionsPage";
import AuditPage from "@/pages/AuditPage";
import OperationsPage from "@/pages/OperationsPage";
import CostPage from "@/pages/CostPage";
import SettingsPage from "@/pages/SettingsPage";
import SkillsPage from "@/pages/SkillsPage";
import KnowledgePage from "@/pages/KnowledgePage";
import ProjectDetailPage from "@/pages/ProjectDetailPage";
import AgentDetailPage from "@/pages/AgentDetailPage";
import ChatPage from "@/pages/ChatPage";

function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const token = useAuthStore((s) => s.token);
  if (!token) return <Navigate to="/login" replace />;
  return <>{children}</>;
}

export default function App() {
  return (
    <Routes>
      <Route path="/login" element={<LoginPage />} />
      <Route
        path="/*"
        element={
          <ProtectedRoute>
            <AppLayout>
              <Routes>
                <Route path="/" element={<ChatPage />} />
                <Route path="/dashboard" element={<DashboardPage />} />
                <Route path="/projects" element={<ProjectsPage />} />
                <Route path="/projects/:name" element={<ProjectDetailPage />} />
                <Route path="/agents" element={<AgentsPage />} />
                <Route path="/agents/:name" element={<AgentDetailPage />} />
                <Route path="/tasks" element={<TasksPage />} />
                <Route path="/missions" element={<MissionsPage />} />
                <Route path="/operations" element={<OperationsPage />} />
                <Route path="/audit" element={<AuditPage />} />
                <Route path="/knowledge" element={<KnowledgePage />} />
                <Route path="/memory" element={<KnowledgePage />} />
                <Route path="/skills" element={<SkillsPage />} />
                <Route path="/blackboard" element={<KnowledgePage />} />
                <Route path="/cost" element={<CostPage />} />
                <Route path="/settings" element={<SettingsPage />} />
              </Routes>
            </AppLayout>
          </ProtectedRoute>
        }
      />
    </Routes>
  );
}
