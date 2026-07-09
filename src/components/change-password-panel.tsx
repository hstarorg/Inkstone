import { useState } from "react";
import { KeyRound } from "lucide-react";
import { Button } from "@/components/ui/button";
import { vaultApi } from "@/lib/vault";

export function ChangePasswordPanel({
  vault,
  onClose,
}: {
  vault: string;
  onClose: () => void;
}) {
  const [currentPassword, setCurrentPassword] = useState("");
  const [newPassword, setNewPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [done, setDone] = useState(false);

  const submit = async () => {
    if (newPassword !== confirmPassword) {
      setError("New passwords do not match.");
      return;
    }
    if (currentPassword.length === 0 || newPassword.length === 0) return;
    setError(null);
    try {
      await vaultApi.changePassword(vault, currentPassword, newPassword);
      setDone(true);
    } catch (changeError) {
      setError(String(changeError));
    }
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-start justify-center bg-black/30 pt-[15vh]"
      onMouseDown={onClose}
    >
      <div
        className="flex w-full max-w-sm flex-col gap-3 rounded-xl border bg-popover p-4 text-popover-foreground shadow-2xl"
        onMouseDown={(event) => event.stopPropagation()}
        onKeyDown={(event) => {
          if (event.key === "Escape") onClose();
          if (event.key === "Enter") void submit();
        }}
      >
        <div className="flex items-center gap-2">
          <KeyRound className="size-4 text-muted-foreground" />
          <h2 className="text-sm font-medium">Change master password</h2>
        </div>
        {done ? (
          <>
            <p className="text-sm text-muted-foreground">
              Password changed. Your recovery code still works and does not need
              to change.
            </p>
            <Button onClick={onClose}>Done</Button>
          </>
        ) : (
          <>
            <input
              type="password"
              autoFocus
              value={currentPassword}
              onChange={(event) => setCurrentPassword(event.target.value)}
              placeholder="Current master password"
              className="rounded-md border bg-transparent px-3 py-1.5 text-sm"
            />
            <div className="my-1 h-px bg-border" />
            <input
              type="password"
              value={newPassword}
              onChange={(event) => setNewPassword(event.target.value)}
              placeholder="New master password"
              className="rounded-md border bg-transparent px-3 py-1.5 text-sm"
            />
            <input
              type="password"
              value={confirmPassword}
              onChange={(event) => setConfirmPassword(event.target.value)}
              placeholder="Confirm new password"
              className="rounded-md border bg-transparent px-3 py-1.5 text-sm"
            />
            <div className="flex justify-end gap-2">
              <Button variant="secondary" onClick={onClose}>
                Cancel
              </Button>
              <Button
                onClick={() => void submit()}
                disabled={
                  currentPassword.length === 0 ||
                  newPassword.length === 0 ||
                  newPassword !== confirmPassword
                }
              >
                Change password
              </Button>
            </div>
          </>
        )}
        {error && <p className="text-xs text-destructive">{error}</p>}
      </div>
    </div>
  );
}
