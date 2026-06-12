//! UI Components for Arniko
//!
//! This module provides a comprehensive set of UI components
//! that can render to HTML strings or native UI elements.

pub mod alert;
pub mod badge;
pub mod button;
pub mod card;
pub mod input;
pub mod kbd;
pub mod metric_card;
pub mod progress_bar;
pub mod separator;
pub mod skeleton;
pub mod spinner;
pub mod status_grid;
pub mod tooltip;

// Re-export all components for convenience
pub use alert::*;
pub use badge::*;
pub use button::*;
pub use card::*;
pub use input::*;
pub use kbd::*;
pub use metric_card::*;
pub use progress_bar::*;
pub use separator::*;
pub use skeleton::*;
pub use spinner::*;
pub use status_grid::*;
pub use tooltip::*;

/// The default Arniko stylesheet. Include this in your HTML `<style>` tag.
pub const ARNIKO_STYLES: &str = r#"
.arniko-btn {
  display: inline-flex; align-items: center; justify-content: center;
  gap: 8px; white-space: nowrap; border-radius: 8px; font-size: 14px;
  font-weight: 500; cursor: pointer; border: none;
  transition: all 0.2s; padding: 8px 16px; height: 36px;
}
.arniko-btn:disabled { opacity: 0.5; pointer-events: none; }
.arniko-btn-default { background: #2563eb; color: #fff; }
.arniko-btn-default:hover { filter: brightness(1.1); }
.arniko-btn-destructive { background: #dc2626; color: #fff; }
.arniko-btn-outline { background: transparent; border: 1px solid #374151; color: #e5e7eb; }
.arniko-btn-outline:hover { background: rgba(255,255,255,0.05); }
.arniko-btn-secondary { background: #374151; color: #f3f4f6; }
.arniko-btn-ghost { background: transparent; color: #e5e7eb; }
.arniko-btn-ghost:hover { background: rgba(255,255,255,0.05); }
.arniko-btn-accent { background: #7c3aed; color: #fff; }
.arniko-btn-sm { height: 32px; padding: 4px 12px; font-size: 12px; }
.arniko-btn-lg { height: 40px; padding: 8px 24px; font-size: 16px; }
.arniko-btn-icon { height: 36px; width: 36px; padding: 0; }

.arniko-card {
  background: #18181b; border: 1px solid #27272a; border-radius: 8px; padding: 20px;
}
.arniko-card-title {
  font-size: 13px; color: #71717a; text-transform: uppercase;
  letter-spacing: 0.5px; margin-bottom: 8px;
}
.arniko-card-body { font-size: 16px; color: #e4e4e7; }

.arniko-input {
  background: #18181b; border: 1px solid #27272a; border-radius: 6px;
  padding: 8px 12px; color: #e4e4e7; font-size: 14px; width: 100%;
}
.arniko-input:focus { outline: none; border-color: #6366f1; }

.arniko-badge {
  display: inline-block; padding: 2px 8px; border-radius: 4px;
  font-size: 12px; font-weight: 500;
}
.arniko-badge-default { background: #27272a; color: #a1a1aa; }
.arniko-badge-success { background: #052e16; color: #22c55e; }
.arniko-badge-warning { background: #422006; color: #f59e0b; }
.arniko-badge-error { background: #450a0a; color: #ef4444; }
.arniko-badge-info { background: #0c1a3d; color: #60a5fa; }
.arniko-badge-purple { background: #2e1065; color: #c4b5fd; }

.arniko-separator { border: none; border-top: 1px solid #27272a; margin: 16px 0; }

.arniko-spinner {
  display: inline-block; width: 20px; height: 20px;
  border: 2px solid #27272a; border-top-color: #6366f1;
  border-radius: 50%; animation: arniko-spin 0.6s linear infinite;
}
@keyframes arniko-spin { to { transform: rotate(360deg); } }

.arniko-kbd {
  background: #27272a; padding: 2px 6px; border-radius: 3px;
  font-size: 12px; font-family: monospace; color: #a1a1aa;
}

/* Metric Cards */
.arniko-metric-card {
  background: #18181b; border: 1px solid #27272a; border-radius: 12px;
  padding: 20px; transition: all 0.2s ease;
}
.arniko-metric-card:hover { border-color: rgba(168, 85, 247, 0.3); }
.arniko-metric-header { display: flex; justify-content: space-between; align-items: flex-start; }
.arniko-metric-info { flex: 1; }
.arniko-metric-title { font-size: 12px; color: #71717a; text-transform: uppercase; letter-spacing: 0.5px; margin-bottom: 4px; }
.arniko-metric-value { font-size: 28px; font-weight: 700; color: #fff; }
.arniko-metric-subtitle { font-size: 12px; color: #52525b; margin-top: 4px; }
.arniko-metric-icon {
  width: 48px; height: 48px; border-radius: 10px; display: flex;
  align-items: center; justify-content: center; font-size: 20px;
}
.arniko-metric-blue { background: linear-gradient(135deg, #3b82f6, #06b6d4); }
.arniko-metric-green { background: linear-gradient(135deg, #22c55e, #10b981); }
.arniko-metric-purple { background: linear-gradient(135deg, #a855f7, #ec4899); }
.arniko-metric-orange { background: linear-gradient(135deg, #f97316, #ef4444); }
.arniko-metric-red { background: linear-gradient(135deg, #ef4444, #dc2626); }
.arniko-metric-cyan { background: linear-gradient(135deg, #06b6d4, #0ea5e9); }
.arniko-metric-footer { margin-top: 16px; font-size: 12px; color: #52525b; }
.arniko-metric-trend { margin-right: 4px; }
.arniko-trend-up { color: #22c55e; }
.arniko-trend-down { color: #ef4444; }
.arniko-trend-stable { color: #eab308; }

/* Progress Bars */
.arniko-progress { margin-bottom: 16px; }
.arniko-progress-header { display: flex; justify-content: space-between; margin-bottom: 8px; }
.arniko-progress-label { font-size: 13px; color: #a1a1aa; }
.arniko-progress-pct { font-size: 13px; color: #71717a; }
.arniko-progress-track { background: #27272a; border-radius: 999px; height: 8px; overflow: hidden; }
.arniko-progress-bar { height: 100%; transition: width 0.3s ease; border-radius: 999px; }
.arniko-progress-accent { background: linear-gradient(90deg, #a855f7, #ec4899); }
.arniko-progress-blue { background: linear-gradient(90deg, #3b82f6, #06b6d4); }
.arniko-progress-green { background: linear-gradient(90deg, #22c55e, #10b981); }
.arniko-progress-purple { background: linear-gradient(90deg, #a855f7, #8b5cf6); }
.arniko-progress-orange { background: linear-gradient(90deg, #f97316, #eab308); }
.arniko-progress-red { background: linear-gradient(90deg, #ef4444, #f97316); }

/* Status Indicators */
.arniko-status-grid { display: grid; gap: 12px; }
.arniko-status-item {
  display: flex; justify-content: space-between; align-items: center;
  padding: 12px; background: rgba(0,0,0,0.3); border-radius: 8px;
}
.arniko-status-label { font-size: 13px; color: #a1a1aa; }
.arniko-status-value { display: flex; align-items: center; gap: 8px; }
.arniko-status-dot { width: 8px; height: 8px; border-radius: 50%; }
.arniko-status-text { font-size: 12px; font-weight: 500; }
.arniko-status-pulse { animation: arniko-pulse 2s ease-in-out infinite; }
@keyframes arniko-pulse { 0%, 100% { opacity: 1; } 50% { opacity: 0.5; } }
.arniko-status-active { background: #22c55e; color: #22c55e; }
.arniko-status-warning { background: #eab308; color: #eab308; }
.arniko-status-error { background: #ef4444; color: #ef4444; }
.arniko-status-idle { background: #6366f1; color: #6366f1; }
.arniko-status-offline { background: #52525b; color: #52525b; }

/* Tooltips */
.arniko-tooltip {
  position: relative; cursor: help;
}
.arniko-tooltip::after {
  content: attr(data-tooltip);
  position: absolute; padding: 6px 10px; border-radius: 6px;
  background: #27272a; color: #e4e4e7; font-size: 12px;
  white-space: nowrap; opacity: 0; pointer-events: none;
  transition: opacity 0.2s; z-index: 1000;
}
.arniko-tooltip:hover::after { opacity: 1; }
.arniko-tooltip[data-tooltip-position="top"]::after { bottom: 100%; left: 50%; transform: translateX(-50%); margin-bottom: 6px; }
.arniko-tooltip[data-tooltip-position="bottom"]::after { top: 100%; left: 50%; transform: translateX(-50%); margin-top: 6px; }
.arniko-tooltip[data-tooltip-position="left"]::after { right: 100%; top: 50%; transform: translateY(-50%); margin-right: 6px; }
.arniko-tooltip[data-tooltip-position="right"]::after { left: 100%; top: 50%; transform: translateY(-50%); margin-left: 6px; }

/* Skeletons */
.arniko-skeleton {
  background: linear-gradient(90deg, #27272a 25%, #3f3f46 50%, #27272a 75%);
  background-size: 200% 100%;
  animation: arniko-shimmer 1.5s infinite;
  border-radius: 4px;
}
@keyframes arniko-shimmer { 0% { background-position: 200% 0; } 100% { background-position: -200% 0; } }
.arniko-skeleton-card {
  background: #18181b; border: 1px solid #27272a; border-radius: 8px; padding: 20px;
}

/* Alerts */
.arniko-alert {
  display: flex; align-items: center; gap: 10px; padding: 12px 16px;
  border-radius: 8px; font-size: 14px;
}
.arniko-alert-icon { font-size: 16px; flex-shrink: 0; }
.arniko-alert-message { flex: 1; }
.arniko-alert-info { background: #172554; border: 1px solid #1e3a8a; color: #60a5fa; }
.arniko-alert-success { background: #052e16; border: 1px solid #166534; color: #22c55e; }
.arniko-alert-warning { background: #422006; border: 1px solid #854d0e; color: #f59e0b; }
.arniko-alert-error { background: #450a0a; border: 1px solid #991b1b; color: #ef4444; }
"#;
