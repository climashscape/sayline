import { AlertIcon, CheckIcon, CloseIcon } from "../Icons";

export type ToastTone = "info" | "success" | "error";

export interface ToastData {
  id: number;
  message: string;
  tone: ToastTone;
}

export function Toast({
  toast,
  onClose,
}: {
  toast: ToastData | null;
  onClose: () => void;
}) {
  if (!toast) return null;
  return (
    <div className={`toast toast-${toast.tone}`} role="status">
      <span className="toast-icon">
        {toast.tone === "error" ? <AlertIcon size={16} /> : <CheckIcon size={16} />}
      </span>
      <span className="toast-message">{toast.message}</span>
      <button className="icon-button toast-close" onClick={onClose} aria-label="关闭提示">
        <CloseIcon size={14} />
      </button>
    </div>
  );
}
