import { invoke } from "@tauri-apps/api/core";
import type { Directory, ScanResult } from "$lib/types";

export const addDirectory = (path: string, recursive = true) =>
  invoke<Directory>("add_directory", { path, recursive });

export const listDirectories = () =>
  invoke<Directory[]>("list_directories");

export const deleteDirectory = (id: number) =>
  invoke<void>("delete_directory", { id });

export const scanDirectory = (id: number) =>
  invoke<ScanResult>("scan_directory", { id });
