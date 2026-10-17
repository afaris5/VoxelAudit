import { invoke, isTauri } from '@tauri-apps/api/core';
import type { Report } from './types';

export const desktop = isTauri();

export async function scan(mode: string): Promise<Report | null> {
  if (!desktop)
    throw new Error(
      'Open the VoxelAudit desktop application to inspect local projects.',
    );
  return invoke<Report | null>('scan', { mode });
}

export interface PreviewData {
  text: string | null;
  image: string | null;
}

export async function preview(path: string): Promise<PreviewData> {
  return invoke<PreviewData>('preview', { path });
}

export async function cancelScan(): Promise<void> {
  return invoke('cancel_scan');
}

export async function exportReport(): Promise<boolean> {
  return invoke<boolean>('export_report');
}
