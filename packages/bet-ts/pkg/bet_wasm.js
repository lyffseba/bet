/* @ts-self-types="./bet_wasm.d.ts" */

class WasmHangman {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmHangmanFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmhangman_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    attempts_left() {
        const ret = wasm.wasmhangman_attempts_left(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {string}
     */
    display_word() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmhangman_display_word(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Guess a letter. Returns true if the letter is in the word.
     * @param {string} ch
     * @returns {boolean}
     */
    guess(ch) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(ch, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
            const len0 = WASM_VECTOR_LEN;
            wasm.wasmhangman_guess(retptr, this.__wbg_ptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return r0 !== 0;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * @returns {string}
     */
    guessed() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmhangman_guessed(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {boolean}
     */
    is_lost() {
        const ret = wasm.wasmhangman_is_lost(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {boolean}
     */
    is_over() {
        const ret = wasm.wasmhangman_is_over(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {boolean}
     */
    is_won() {
        const ret = wasm.wasmhangman_is_won(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {number}
     */
    max_attempts() {
        const ret = wasm.wasmhangman_max_attempts(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * `words_json`: JSON array of strings, e.g. `["ALPHA","BRAVO"]`.
     * @param {bigint} seed
     * @param {string} words_json
     * @param {number} max_attempts
     */
    constructor(seed, words_json, max_attempts) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(words_json, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
            const len0 = WASM_VECTOR_LEN;
            wasm.wasmhangman_new(retptr, seed, ptr0, len0, max_attempts);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            this.__wbg_ptr = r0 >>> 0;
            WasmHangmanFinalization.register(this, this.__wbg_ptr, this);
            return this;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * @returns {string}
     */
    state_hash() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmhangman_state_hash(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Secret word (use only after game over for UI).
     * @returns {string}
     */
    word() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmhangman_word(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export(deferred1_0, deferred1_1, 1);
        }
    }
}
if (Symbol.dispose) WasmHangman.prototype[Symbol.dispose] = WasmHangman.prototype.free;
exports.WasmHangman = WasmHangman;

class WasmLedger {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmLedgerFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmledger_free(ptr, 0);
    }
    /**
     * @param {string} id
     * @returns {number}
     */
    balance(id) {
        const ptr0 = passStringToWasm0(id, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmledger_balance(this.__wbg_ptr, ptr0, len0);
        return ret;
    }
    /**
     * JSON object map of balances.
     * @returns {string}
     */
    balances_json() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmledger_balances_json(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @param {string} id
     */
    ensure_player(id) {
        const ptr0 = passStringToWasm0(id, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len0 = WASM_VECTOR_LEN;
        wasm.wasmledger_ensure_player(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @param {number} default_grant
     */
    constructor(default_grant) {
        const ret = wasm.wasmledger_new(default_grant);
        this.__wbg_ptr = ret >>> 0;
        WasmLedgerFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @param {string} match_id
     * @returns {number}
     */
    open_pot(match_id) {
        const ptr0 = passStringToWasm0(match_id, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmledger_open_pot(this.__wbg_ptr, ptr0, len0);
        return ret;
    }
    /**
     * @param {string} match_id
     * @param {string | null} [winner]
     * @returns {number}
     */
    settle(match_id, winner) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(match_id, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
            const len0 = WASM_VECTOR_LEN;
            var ptr1 = isLikeNone(winner) ? 0 : passStringToWasm0(winner, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
            var len1 = WASM_VECTOR_LEN;
            wasm.wasmledger_settle(retptr, this.__wbg_ptr, ptr0, len0, ptr1, len1);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return r0;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * @param {string} match_id
     * @param {string} player
     * @param {number} amount
     */
    stake(match_id, player, amount) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(match_id, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
            const len0 = WASM_VECTOR_LEN;
            const ptr1 = passStringToWasm0(player, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
            const len1 = WASM_VECTOR_LEN;
            wasm.wasmledger_stake(retptr, this.__wbg_ptr, ptr0, len0, ptr1, len1, amount);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
}
if (Symbol.dispose) WasmLedger.prototype[Symbol.dispose] = WasmLedger.prototype.free;
exports.WasmLedger = WasmLedger;

class WasmTtt {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmTttFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmttt_free(ptr, 0);
    }
    /**
     * 9-char board: `.` empty, `X`, `O`.
     * @returns {string}
     */
    board() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmttt_board(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * `x` | `o`
     * @returns {string}
     */
    current() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmttt_current(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Human (X) places at index 0..8; AI (O) replies if ongoing.
     * Returns false if the move was illegal.
     * @param {number} index
     * @returns {boolean}
     */
    make_move_vs_ai(index) {
        const ret = wasm.wasmttt_make_move_vs_ai(this.__wbg_ptr, index);
        return ret !== 0;
    }
    /**
     * @param {bigint} seed
     */
    constructor(seed) {
        const ret = wasm.wasmttt_new(seed);
        this.__wbg_ptr = ret >>> 0;
        WasmTttFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Pure multiplayer place: "x" | "o" at index.
     * @param {string} player
     * @param {number} index
     * @returns {boolean}
     */
    place(player, index) {
        const ptr0 = passStringToWasm0(player, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmttt_place(this.__wbg_ptr, ptr0, len0, index);
        return ret !== 0;
    }
    reset() {
        wasm.wasmttt_reset(this.__wbg_ptr);
    }
    /**
     * @returns {string}
     */
    state_hash() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmttt_state_hash(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * `ongoing` | `win_x` | `win_o` | `draw`
     * @returns {string}
     */
    status() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmttt_status(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export(deferred1_0, deferred1_1, 1);
        }
    }
}
if (Symbol.dispose) WasmTtt.prototype[Symbol.dispose] = WasmTtt.prototype.free;
exports.WasmTtt = WasmTtt;

/**
 * @returns {string}
 */
function engine_version() {
    let deferred1_0;
    let deferred1_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.engine_version(retptr);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        deferred1_0 = r0;
        deferred1_1 = r1;
        return getStringFromWasm0(r0, r1);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
        wasm.__wbindgen_export(deferred1_0, deferred1_1, 1);
    }
}
exports.engine_version = engine_version;

/**
 * Deterministic hangman hash path for cross-language goldens.
 * @param {bigint} seed
 * @param {string} words_json
 * @returns {string}
 */
function golden_hangman_hash(seed, words_json) {
    let deferred3_0;
    let deferred3_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(words_json, wasm.__wbindgen_export2, wasm.__wbindgen_export3);
        const len0 = WASM_VECTOR_LEN;
        wasm.golden_hangman_hash(retptr, seed, ptr0, len0);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        var ptr2 = r0;
        var len2 = r1;
        if (r3) {
            ptr2 = 0; len2 = 0;
            throw takeObject(r2);
        }
        deferred3_0 = ptr2;
        deferred3_1 = len2;
        return getStringFromWasm0(ptr2, len2);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
        wasm.__wbindgen_export(deferred3_0, deferred3_1, 1);
    }
}
exports.golden_hangman_hash = golden_hangman_hash;

/**
 * @param {bigint} seed
 * @returns {string}
 */
function golden_ttt_hash(seed) {
    let deferred1_0;
    let deferred1_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.golden_ttt_hash(retptr, seed);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        deferred1_0 = r0;
        deferred1_1 = r1;
        return getStringFromWasm0(r0, r1);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
        wasm.__wbindgen_export(deferred1_0, deferred1_1, 1);
    }
}
exports.golden_ttt_hash = golden_ttt_hash;

function wasm_start() {
    wasm.wasm_start();
}
exports.wasm_start = wasm_start;

function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg___wbindgen_throw_6ddd609b62940d55: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbindgen_cast_0000000000000001: function(arg0, arg1) {
            // Cast intrinsic for `Ref(String) -> Externref`.
            const ret = getStringFromWasm0(arg0, arg1);
            return addHeapObject(ret);
        },
        __wbindgen_object_drop_ref: function(arg0) {
            takeObject(arg0);
        },
    };
    return {
        __proto__: null,
        "./bet_wasm_bg.js": import0,
    };
}

const WasmHangmanFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmhangman_free(ptr >>> 0, 1));
const WasmLedgerFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmledger_free(ptr >>> 0, 1));
const WasmTttFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmttt_free(ptr >>> 0, 1));

function addHeapObject(obj) {
    if (heap_next === heap.length) heap.push(heap.length + 1);
    const idx = heap_next;
    heap_next = heap[idx];

    heap[idx] = obj;
    return idx;
}

function dropObject(idx) {
    if (idx < 1028) return;
    heap[idx] = heap_next;
    heap_next = idx;
}

let cachedDataViewMemory0 = null;
function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return decodeText(ptr, len);
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function getObject(idx) { return heap[idx]; }

let heap = new Array(1024).fill(undefined);
heap.push(undefined, null, true, false);

let heap_next = heap.length;

function isLikeNone(x) {
    return x === undefined || x === null;
}

function passStringToWasm0(arg, malloc, realloc) {
    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }
    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = cachedTextEncoder.encodeInto(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function takeObject(idx) {
    const ret = getObject(idx);
    dropObject(idx);
    return ret;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
function decodeText(ptr, len) {
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    };
}

let WASM_VECTOR_LEN = 0;

const wasmPath = `${__dirname}/bet_wasm_bg.wasm`;
const wasmBytes = require('fs').readFileSync(wasmPath);
const wasmModule = new WebAssembly.Module(wasmBytes);
let wasm = new WebAssembly.Instance(wasmModule, __wbg_get_imports()).exports;
wasm.__wbindgen_start();
