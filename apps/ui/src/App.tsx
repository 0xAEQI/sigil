import { Routes, Route, Navigate } from "react-router-dom";
import { useAuthStore } from "@/store/auth";
import AppLayout from "@/components/AppLayout";
import LoginPage from "@/pages/LoginPage";
import DashboardPage from "@/pages/DashboardPage";
import SessionsPage from "@/pages/SessionsPage";
import TasksPage from "@/pages/TasksPage";
import DepartmentsPage from "@/pages/DepartmentsPage";
import MemoryPage from "@/pages/MemoryPage";
import TriggersPage from "@/pages/TriggersPage";
import SkillsPage from "@/pages/SkillsPage";
import FinancePage from "@/pages/FinancePage";

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
        path="/"
        element={
          <ProtectedRoute>
            <AppLayout />
          </ProtectedRoute>
        }
      >
        <Route index element={<DashboardPage />} />
        <Route path="sessions" element={<SessionsPage />} />
        <Route path="issues" element={<TasksPage />} />
        <Route path="triggers" element={<TriggersPage />} />
        <Route path="skills" element={<SkillsPage />} />
        <Route path="memories" element={<MemoryPage />} />
        <Route path="notes" element={<MemoryPage />} />
        <Route path="finance" element={<FinancePage />} />
        <Route path="departments/:id" element={<DepartmentsPage />} />

        {/* Redirects */}
        <Route path="inbox" element={<Navigate to="/sessions" replace />} />
        <Route path="tasks" element={<Navigate to="/issues" replace />} />
        <Route path="automations" element={<Navigate to="/triggers" replace />} />
        <Route path="knowledge" element={<Navigate to="/memories" replace />} />
        <Route path="memory" element={<Navigate to="/memories" replace />} />
        <Route path="notes" element={<Navigate to="/memories" replace />} />
        <Route path="blackboard" element={<Navigate to="/memories" replace />} />
        <Route path="cost" element={<Navigate to="/finance" replace />} />
        <Route path="audit" element={<Navigate to="/" replace />} />
        <Route path="dashboard" element={<Navigate to="/" replace />} />
        <Route path="agents" element={<Navigate to="/" replace />} />
        <Route path="settings" element={<Navigate to="/" replace />} />
      </Route>
    </Routes>
  );
}
