import { useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { useAuthStore } from "@/store/auth";

export default function AuthCallbackPage() {
  const navigate = useNavigate();
  const handleOAuthCallback = useAuthStore((s) => s.handleOAuthCallback);

  useEffect(() => {
    // Token comes in the URL hash fragment: /#/auth/callback?token=JWT
    const params = new URLSearchParams(window.location.hash.split("?")[1] || "");
    const token = params.get("token");

    if (token) {
      handleOAuthCallback(token);
      navigate("/", { replace: true });
    } else {
      navigate("/login", { replace: true });
    }
  }, [handleOAuthCallback, navigate]);

  return (
    <div className="login-page">
      <div className="login-card">
        <p style={{ color: "var(--text-muted)", fontSize: 13, textAlign: "center" }}>
          Signing you in...
        </p>
      </div>
    </div>
  );
}
