// Application-wide reactive state stores
import { writable } from 'svelte/store';

/** @type {import('svelte/store').Writable<SystemStatus|null>} */
export const system = writable(null);

/** @type {import('svelte/store').Writable<FileItem[]>} */
export const queue  = writable([]);

/** @type {import('svelte/store').Writable<LogEntry[]>} */
export const log    = writable([]);

export const isConverting  = writable(false);
export const totalProgress = writable(0); // 0..1 overall batch progress

// Conversion settings
export const convType    = writable('AudioMP3');   // AudioMP3 | AudioAAC | VideoMKV | VideoH264 | VideoH265
export const acceleration = writable('None');      // None | NVENC | QSV | AMF | VAAPI | VideoToolbox
export const preserveGrain = writable(false);
export const optimizeColor = writable(false);
export const outputDir   = writable(null);         // null = default (source folder)
export const youtubeUrl  = writable('');

/**
 * @typedef {Object} FileItem
 * @property {string}  id
 * @property {string}  path
 * @property {string}  name
 * @property {boolean} selected
 * @property {string}  status   - 'pending' | 'converting' | 'done' | 'error'
 * @property {number}  progress - 0..1
 * @property {string}  [message]
 * @property {string}  [youtubeUrl]
 * @property {string}  [container]
 * @property {string}  [vCodec]
 * @property {string}  [aCodec]
 * @property {AudioTrack[]} tracks
 * @property {number}  selectedTrack
 */

/**
 * @typedef {Object} AudioTrack
 * @property {number} stream_index
 * @property {string} codec
 * @property {string} language
 */

/**
 * @typedef {Object} LogEntry
 * @property {boolean} ok
 * @property {string}  msg
 * @property {number}  ts
 */

/**
 * @typedef {Object} SystemStatus
 * @property {boolean} ffmpeg_ok
 * @property {string}  ffmpeg_version
 * @property {boolean} ytdlp_ok
 * @property {string}  ytdlp_version
 * @property {boolean} hw_nvenc
 * @property {boolean} hw_qsv
 * @property {boolean} hw_amf
 * @property {boolean} hw_vaapi
 * @property {boolean} hw_vtb
 */
