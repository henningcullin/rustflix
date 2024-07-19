import { Directory } from '@/directories/Directories';
import { Film } from '@/films/Films';
import { atom } from "jotai";

export const directoryAtom = atom<Directory[]>([]);

export const filmAtom = atom<Film[]>([]);