import { useState, useEffect, useRef } from "react";
import CompanyPatternIcon from "./CompanyPatternIcon";
import { api } from "@/lib/api";
import { useDaemonStore } from "@/store/daemon";
import { useChatStore } from "@/store/chat";

interface Props {
  open: boolean;
  onClose: () => void;
}

export default function CreateCompanyModal({ open, onClose }: Props) {
  const [name, setName] = useState("");
  const [creating, setCreating] = useState(false);
  const [error, setError] = useState("");
  const inputRef = useRef<HTMLInputElement>(null);
  const fetchCompanies = useDaemonStore((s) => s.fetchCompanies);
  const setChannel = useChatStore((s) => s.setChannel);

  useEffect(() => {
    if (open) {
      setName("");
      setError("");
      setCreating(false);
      setTimeout(() => inputRef.current?.focus(), 50);
    }
  }, [open]);

  useEffect(() => {
    if (!open) return;
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [open, onClose]);

  const handleCreate = async () => {
    const trimmed = name.trim();
    if (!trimmed) return;
    setCreating(true);
    setError("");
    try {
      await api.createCompany({ name: trimmed });
      await fetchCompanies();
      setChannel(trimmed);
      onClose();
    } catch (err: any) {
      setError(err?.message || "Failed to create company");
    } finally {
      setCreating(false);
    }
  };

  if (!open) return null;

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="create-company-modal" onClick={(e) => e.stopPropagation()}>
        <div className="create-company-header">New Company</div>
        <div className="create-company-preview">
          <CompanyPatternIcon name={name || "New"} selected />
          {name.trim() && (
            <span className="create-company-preview-name">{name.trim()}</span>
          )}
        </div>
        <input
          ref={inputRef}
          className="create-company-input"
          type="text"
          placeholder="Company name"
          value={name}
          onChange={(e) => setName(e.target.value)}
          onKeyDown={(e) => { if (e.key === "Enter") handleCreate(); }}
          maxLength={40}
        />
        {error && <div className="create-company-error">{error}</div>}
        <div className="create-company-actions">
          <button className="create-company-btn cancel" onClick={onClose}>Cancel</button>
          <button
            className="create-company-btn create"
            onClick={handleCreate}
            disabled={!name.trim() || creating}
          >
            {creating ? "Creating..." : "Create"}
          </button>
        </div>
      </div>
    </div>
  );
}
