import { Directory } from '@/directories/Directories';
import { atom } from "jotai";
import { Film } from './types';

export const directoryAtom = atom<Directory[]>([]);

export const filmAtom = atom<Film[]>([]);