import type { LucideIcon } from "lucide-react";
import { useTranslation } from "react-i18next";
import { formatTokens } from "../lib/format";

interface Props {
  label: string;
  value: number;
  icon: LucideIcon;
  tone: "blue" | "green" | "amber" | "rose";
}

export function Metric({ label, value, icon: Icon, tone }: Props) {
  const { i18n } = useTranslation();
  return (
    <div className="metric">
      <div className={`metric-icon ${tone}`}><Icon size={18} /></div>
      <div><span>{label}</span><strong>{formatTokens(value, i18n.resolvedLanguage ?? i18n.language)}</strong></div>
    </div>
  );
}
