import { Directory } from '@/directories/Directories';
import { atom } from "jotai";

export const directoryAtom = atom<Directory[]>([]);