
let wasm;

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });

cachedTextDecoder.decode();

let cachegetUint8Memory0 = null;
function getUint8Memory0() {
    if (cachegetUint8Memory0 === null || cachegetUint8Memory0.buffer !== wasm.memory.buffer) {
        cachegetUint8Memory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachegetUint8Memory0;
}

function getStringFromWasm0(ptr, len) {
    return cachedTextDecoder.decode(getUint8Memory0().subarray(ptr, ptr + len));
}

const heap = new Array(32).fill(undefined);

heap.push(undefined, null, true, false);

let heap_next = heap.length;

function addHeapObject(obj) {
    if (heap_next === heap.length) heap.push(heap.length + 1);
    const idx = heap_next;
    heap_next = heap[idx];

    heap[idx] = obj;
    return idx;
}

function getObject(idx) { return heap[idx]; }

function dropObject(idx) {
    if (idx < 36) return;
    heap[idx] = heap_next;
    heap_next = idx;
}

function takeObject(idx) {
    const ret = getObject(idx);
    dropObject(idx);
    return ret;
}

let WASM_VECTOR_LEN = 0;

let cachedTextEncoder = new TextEncoder('utf-8');

const encodeString = (typeof cachedTextEncoder.encodeInto === 'function'
    ? function (arg, view) {
    return cachedTextEncoder.encodeInto(arg, view);
}
    : function (arg, view) {
    const buf = cachedTextEncoder.encode(arg);
    view.set(buf);
    return {
        read: arg.length,
        written: buf.length
    };
});

function passStringToWasm0(arg, malloc, realloc) {

    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length);
        getUint8Memory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len);

    const mem = getUint8Memory0();

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
        ptr = realloc(ptr, len, len = offset + arg.length * 3);
        const view = getUint8Memory0().subarray(ptr + offset, ptr + len);
        const ret = encodeString(arg, view);

        offset += ret.written;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

let cachegetInt32Memory0 = null;
function getInt32Memory0() {
    if (cachegetInt32Memory0 === null || cachegetInt32Memory0.buffer !== wasm.memory.buffer) {
        cachegetInt32Memory0 = new Int32Array(wasm.memory.buffer);
    }
    return cachegetInt32Memory0;
}

function debugString(val) {
    // primitive types
    const type = typeof val;
    if (type == 'number' || type == 'boolean' || val == null) {
        return  `${val}`;
    }
    if (type == 'string') {
        return `"${val}"`;
    }
    if (type == 'symbol') {
        const description = val.description;
        if (description == null) {
            return 'Symbol';
        } else {
            return `Symbol(${description})`;
        }
    }
    if (type == 'function') {
        const name = val.name;
        if (typeof name == 'string' && name.length > 0) {
            return `Function(${name})`;
        } else {
            return 'Function';
        }
    }
    // objects
    if (Array.isArray(val)) {
        const length = val.length;
        let debug = '[';
        if (length > 0) {
            debug += debugString(val[0]);
        }
        for(let i = 1; i < length; i++) {
            debug += ', ' + debugString(val[i]);
        }
        debug += ']';
        return debug;
    }
    // Test for built-in
    const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
    let className;
    if (builtInMatches.length > 1) {
        className = builtInMatches[1];
    } else {
        // Failed to match the standard '[object ClassName]'
        return toString.call(val);
    }
    if (className == 'Object') {
        // we're a user defined class or Object
        // JSON.stringify avoids problems with cycles, and is generally much
        // easier than looping through ownProperties of `val`.
        try {
            return 'Object(' + JSON.stringify(val) + ')';
        } catch (_) {
            return 'Object';
        }
    }
    // errors
    if (val instanceof Error) {
        return `${val.name}: ${val.message}\n${val.stack}`;
    }
    // TODO we could test for more things here, like `Set`s and `Map`s.
    return className;
}

function makeMutClosure(arg0, arg1, dtor, f) {
    const state = { a: arg0, b: arg1, cnt: 1, dtor };
    const real = (...args) => {
        // First up with a closure we increment the internal reference
        // count. This ensures that the Rust closure environment won't
        // be deallocated while we're invoking it.
        state.cnt++;
        const a = state.a;
        state.a = 0;
        try {
            return f(a, state.b, ...args);
        } finally {
            if (--state.cnt === 0) {
                wasm.__wbindgen_export_2.get(state.dtor)(a, state.b);

            } else {
                state.a = a;
            }
        }
    };
    real.original = state;

    return real;
}
function __wbg_adapter_30(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hbb4ab9b66a2c7a2d(arg0, arg1, arg2);
}

function __wbg_adapter_33(arg0, arg1) {
    var ret = wasm._dyn_core__ops__function__FnMut_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hf61483a107f788f8(arg0, arg1);
    return takeObject(ret);
}

function __wbg_adapter_36(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h51cdae26a6f5551b(arg0, arg1, addHeapObject(arg2));
}

let stack_pointer = 32;

function addBorrowedObject(obj) {
    if (stack_pointer == 1) throw new Error('out of js stack');
    heap[--stack_pointer] = obj;
    return stack_pointer;
}
function __wbg_adapter_39(arg0, arg1, arg2) {
    try {
        wasm._dyn_core__ops__function__FnMut___A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h1b5773ced7ef7ead(arg0, arg1, addBorrowedObject(arg2));
    } finally {
        heap[stack_pointer++] = undefined;
    }
}

function __wbg_adapter_42(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hea81d12c5fbfa36a(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_45(arg0, arg1) {
    wasm._dyn_core__ops__function__FnMut_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hbf08d823ac5dfe5a(arg0, arg1);
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        wasm.__wbindgen_exn_store(addHeapObject(e));
    }
}

function getArrayU8FromWasm0(ptr, len) {
    return getUint8Memory0().subarray(ptr / 1, ptr / 1 + len);
}
function __wbg_adapter_88(arg0, arg1, arg2, arg3, arg4) {
    wasm.wasm_bindgen__convert__closures__invoke3_mut__h8aed6d58ed6ff963(arg0, arg1, addHeapObject(arg2), arg3, addHeapObject(arg4));
}

let cachegetUint32Memory0 = null;
function getUint32Memory0() {
    if (cachegetUint32Memory0 === null || cachegetUint32Memory0.buffer !== wasm.memory.buffer) {
        cachegetUint32Memory0 = new Uint32Array(wasm.memory.buffer);
    }
    return cachegetUint32Memory0;
}

function getArrayJsValueFromWasm0(ptr, len) {
    const mem = getUint32Memory0();
    const slice = mem.subarray(ptr / 4, ptr / 4 + len);
    const result = [];
    for (let i = 0; i < slice.length; i++) {
        result.push(takeObject(slice[i]));
    }
    return result;
}

async function load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);

            } catch (e) {
                if (module.headers.get('Content-Type') != 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else {
                    throw e;
                }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);

    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };

        } else {
            return instance;
        }
    }
}

async function init(input) {
    if (typeof input === 'undefined') {
        input = new URL('index-fec749d0b9ebf005_bg.wasm', import.meta.url);
    }
    const imports = {};
    imports.wbg = {};
    imports.wbg.__wbindgen_json_parse = function(arg0, arg1) {
        var ret = JSON.parse(getStringFromWasm0(arg0, arg1));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_new_bc0197e4a3626208 = function(arg0, arg1, arg2) {
        var ret = new L.Map(getStringFromWasm0(arg0, arg1), getObject(arg2));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_new_36006d744e61fd23 = function() {
        var ret = new L.LayerGroup();
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_addTo_395b65234e63d6aa = function(arg0, arg1) {
        getObject(arg0).addTo(getObject(arg1));
    };
    imports.wbg.__wbindgen_string_new = function(arg0, arg1) {
        var ret = getStringFromWasm0(arg0, arg1);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_object_drop_ref = function(arg0) {
        takeObject(arg0);
    };
    imports.wbg.__wbg_extend_2c189a4bf6fa5d7d = function(arg0) {
        var ret = L.Control.extend(getObject(arg0));
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_is_function = function(arg0) {
        var ret = typeof(getObject(arg0)) === 'function';
        return ret;
    };
    imports.wbg.__wbg_new_ec75d0d5815be736 = function() {
        var ret = new Array();
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_construct_46315653ee8f12df = function() { return handleError(function (arg0, arg1) {
        var ret = Reflect.construct(getObject(arg0), getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_addTo_c671552024326965 = function(arg0, arg1) {
        getObject(arg0).addTo(getObject(arg1));
    };
    imports.wbg.__wbg_new_41e255d4145eb897 = function(arg0, arg1, arg2) {
        var ret = new L.TileLayer(getStringFromWasm0(arg0, arg1), getObject(arg2));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_addTo_ff768129a27669b2 = function(arg0, arg1) {
        getObject(arg0).addTo(getObject(arg1));
    };
    imports.wbg.__wbg_coords_742abdbddec0be0d = function(arg0) {
        var ret = getObject(arg0).coords;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_latitude_e8c96ede166cddd5 = function(arg0) {
        var ret = getObject(arg0).latitude;
        return ret;
    };
    imports.wbg.__wbg_longitude_da8b486bbeff5152 = function(arg0) {
        var ret = getObject(arg0).longitude;
        return ret;
    };
    imports.wbg.__wbg_document_29fb71d7cea23553 = function(arg0) {
        var ret = getObject(arg0).document;
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    };
    imports.wbg.__wbg_setclassName_e8c8e60b9841495d = function(arg0, arg1, arg2) {
        getObject(arg0).className = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_instanceof_HtmlAnchorElement_58dba5e28e1b79c7 = function(arg0) {
        var ret = getObject(arg0) instanceof HTMLAnchorElement;
        return ret;
    };
    imports.wbg.__wbg_sethref_e391175f5b33f56c = function(arg0, arg1, arg2) {
        getObject(arg0).href = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_setinnerHTML_5779e0f1b53c018b = function(arg0, arg1, arg2) {
        getObject(arg0).innerHTML = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_settitle_47b93f90acf7aa5a = function(arg0, arg1, arg2) {
        getObject(arg0).title = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_new_4b48f9f8159fea77 = function() {
        var ret = new Object();
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_addEventListener_1187d9892750204d = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).addEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3), getObject(arg4));
    }, arguments) };
    imports.wbg.__wbindgen_object_clone_ref = function(arg0) {
        var ret = getObject(arg0);
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_removeEventListener_cb28b2b1c2746766 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).removeEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3));
    }, arguments) };
    imports.wbg.__wbindgen_cb_drop = function(arg0) {
        const obj = takeObject(arg0).original;
        if (obj.cnt-- == 1) {
            obj.a = 0;
            return true;
        }
        var ret = false;
        return ret;
    };
    imports.wbg.__wbg_clearInterval_a2b667ff2708e7b1 = function(arg0, arg1) {
        getObject(arg0).clearInterval(arg1);
    };
    imports.wbg.__wbg_clearInterval_b21768166ea4faee = function(arg0, arg1) {
        getObject(arg0).clearInterval(arg1);
    };
    imports.wbg.__wbg_cancelAnimationFrame_ac2f82caea825778 = function() { return handleError(function (arg0, arg1) {
        getObject(arg0).cancelAnimationFrame(arg1);
    }, arguments) };
    imports.wbg.__wbg_settextContent_68b1b04017e68705 = function(arg0, arg1, arg2) {
        getObject(arg0).textContent = arg1 === 0 ? undefined : getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_instanceof_Element_4a2e770275c12af2 = function(arg0) {
        var ret = getObject(arg0) instanceof Element;
        return ret;
    };
    imports.wbg.__wbg_performance_bfc7f78eac37a2ef = function(arg0) {
        var ret = getObject(arg0).performance;
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    };
    imports.wbg.__wbg_now_885ca88fafee0fd1 = function(arg0) {
        var ret = getObject(arg0).now();
        return ret;
    };
    imports.wbg.__wbg_requestAnimationFrame_4614b7cdb59d7a59 = function() { return handleError(function (arg0, arg1) {
        var ret = getObject(arg0).requestAnimationFrame(getObject(arg1));
        return ret;
    }, arguments) };
    imports.wbg.__wbg_getElementById_8ef24477d541ac87 = function(arg0, arg1, arg2) {
        var ret = getObject(arg0).getElementById(getStringFromWasm0(arg1, arg2));
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    };
    imports.wbg.__wbg_firstChild_63cd6fc4453e50ff = function(arg0) {
        var ret = getObject(arg0).firstChild;
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    };
    imports.wbg.__wbg_navigator_f92994fb618b3c03 = function(arg0) {
        var ret = getObject(arg0).navigator;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_geolocation_d363e04faff3c30b = function() { return handleError(function (arg0) {
        var ret = getObject(arg0).geolocation;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_watchPosition_6d191fae2d3b36e1 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        var ret = getObject(arg0).watchPosition(getObject(arg1), getObject(arg2), getObject(arg3));
        return ret;
    }, arguments) };
    imports.wbg.__wbindgen_jsval_eq = function(arg0, arg1) {
        var ret = getObject(arg0) === getObject(arg1);
        return ret;
    };
    imports.wbg.__wbg_setInterval_b2fdce77432bdc25 = function() { return handleError(function (arg0, arg1, arg2) {
        var ret = getObject(arg0).setInterval(getObject(arg1), arg2);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_setInterval_05c0a14949da4fec = function() { return handleError(function (arg0, arg1, arg2) {
        var ret = getObject(arg0).setInterval(getObject(arg1), arg2);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_getItem_dca05ca17d9c222d = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        var ret = getObject(arg1).getItem(getStringFromWasm0(arg2, arg3));
        var ptr0 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    }, arguments) };
    imports.wbg.__wbg_preventDefault_7a8483e2b1a95a37 = function(arg0) {
        getObject(arg0).preventDefault();
    };
    imports.wbg.__wbg_history_b841f27e73887d3a = function() { return handleError(function (arg0) {
        var ret = getObject(arg0).history;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_new_8b15d3bc6db879cd = function() { return handleError(function () {
        var ret = new URLSearchParams();
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_append_a1f2bc6af2787968 = function(arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).append(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
    };
    imports.wbg.__wbg_toString_e6bf737ea14c7939 = function(arg0) {
        var ret = getObject(arg0).toString();
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_setsearch_f0f398681e7fbab4 = function(arg0, arg1, arg2) {
        getObject(arg0).search = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_sethash_216ec3a0988de683 = function(arg0, arg1, arg2) {
        getObject(arg0).hash = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_href_0a12b470b5ea3f7a = function(arg0, arg1) {
        var ret = getObject(arg1).href;
        var ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    };
    imports.wbg.__wbg_pushState_b0838d2cad87d27e = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
        getObject(arg0).pushState(getObject(arg1), getStringFromWasm0(arg2, arg3), arg4 === 0 ? undefined : getStringFromWasm0(arg4, arg5));
    }, arguments) };
    imports.wbg.__wbg_target_8eac1b471e53114d = function(arg0) {
        var ret = getObject(arg0).target;
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    };
    imports.wbg.__wbg_instanceof_HtmlInputElement_80e9098b1138bf4b = function(arg0) {
        var ret = getObject(arg0) instanceof HTMLInputElement;
        return ret;
    };
    imports.wbg.__wbg_value_19dfa22a5c5f8a0e = function(arg0, arg1) {
        var ret = getObject(arg1).value;
        var ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    };
    imports.wbg.__wbg_instanceof_HtmlTextAreaElement_769f68e5b765e47d = function(arg0) {
        var ret = getObject(arg0) instanceof HTMLTextAreaElement;
        return ret;
    };
    imports.wbg.__wbg_value_10e481fdf113f964 = function(arg0, arg1) {
        var ret = getObject(arg1).value;
        var ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    };
    imports.wbg.__wbg_instanceof_HtmlSelectElement_c2059deb37971fe4 = function(arg0) {
        var ret = getObject(arg0) instanceof HTMLSelectElement;
        return ret;
    };
    imports.wbg.__wbg_value_d2c86eee29a5b37b = function(arg0, arg1) {
        var ret = getObject(arg1).value;
        var ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    };
    imports.wbg.__wbg_instanceof_HtmlProgressElement_441dae8fae58c35f = function(arg0) {
        var ret = getObject(arg0) instanceof HTMLProgressElement;
        return ret;
    };
    imports.wbg.__wbg_value_7c429847263d9a5e = function(arg0) {
        var ret = getObject(arg0).value;
        return ret;
    };
    imports.wbg.__wbg_instanceof_HtmlOptionElement_bc7fe39d719a1e1f = function(arg0) {
        var ret = getObject(arg0) instanceof HTMLOptionElement;
        return ret;
    };
    imports.wbg.__wbg_value_3c2fa3f161e7a12d = function(arg0, arg1) {
        var ret = getObject(arg1).value;
        var ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    };
    imports.wbg.__wbg_instanceof_HtmlButtonElement_b1e8781bae27ebaf = function(arg0) {
        var ret = getObject(arg0) instanceof HTMLButtonElement;
        return ret;
    };
    imports.wbg.__wbg_value_7613f51d342269f6 = function(arg0, arg1) {
        var ret = getObject(arg1).value;
        var ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    };
    imports.wbg.__wbg_instanceof_HtmlDataElement_2fe30ad352172c12 = function(arg0) {
        var ret = getObject(arg0) instanceof HTMLDataElement;
        return ret;
    };
    imports.wbg.__wbg_value_9a953972112cdab3 = function(arg0, arg1) {
        var ret = getObject(arg1).value;
        var ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    };
    imports.wbg.__wbg_instanceof_HtmlMeterElement_5fe545e1e30a5a69 = function(arg0) {
        var ret = getObject(arg0) instanceof HTMLMeterElement;
        return ret;
    };
    imports.wbg.__wbg_value_dfd3f8f62edb19df = function(arg0) {
        var ret = getObject(arg0).value;
        return ret;
    };
    imports.wbg.__wbg_instanceof_HtmlLiElement_be19f62ed15e9c81 = function(arg0) {
        var ret = getObject(arg0) instanceof HTMLLIElement;
        return ret;
    };
    imports.wbg.__wbg_value_355545340a60ea49 = function(arg0) {
        var ret = getObject(arg0).value;
        return ret;
    };
    imports.wbg.__wbg_instanceof_HtmlOutputElement_1fb66ecf4995fb89 = function(arg0) {
        var ret = getObject(arg0) instanceof HTMLOutputElement;
        return ret;
    };
    imports.wbg.__wbg_value_595a27157ee796d2 = function(arg0, arg1) {
        var ret = getObject(arg1).value;
        var ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    };
    imports.wbg.__wbg_instanceof_HtmlParamElement_dbeba7093e6d96ac = function(arg0) {
        var ret = getObject(arg0) instanceof HTMLParamElement;
        return ret;
    };
    imports.wbg.__wbg_value_e1042dc01d1117c7 = function(arg0, arg1) {
        var ret = getObject(arg1).value;
        var ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    };
    imports.wbg.__wbg_createElementNS_7a572d6c359358bc = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
        var ret = getObject(arg0).createElementNS(arg1 === 0 ? undefined : getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_instanceof_HtmlElement_b7ca85c835f5fa1b = function(arg0) {
        var ret = getObject(arg0) instanceof HTMLElement;
        return ret;
    };
    imports.wbg.__wbg_focus_80a6e6cc38b4cf70 = function() { return handleError(function (arg0) {
        getObject(arg0).focus();
    }, arguments) };
    imports.wbg.__wbg_setvalue_44ece978ee932299 = function(arg0, arg1, arg2) {
        getObject(arg0).value = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_closest_f552aad27fd9b2a6 = function() { return handleError(function (arg0, arg1, arg2) {
        var ret = getObject(arg0).closest(getStringFromWasm0(arg1, arg2));
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_instanceof_PopStateEvent_7351b5b08c2b1932 = function(arg0) {
        var ret = getObject(arg0) instanceof PopStateEvent;
        return ret;
    };
    imports.wbg.__wbg_state_11cf8ac1c3653a7e = function(arg0) {
        var ret = getObject(arg0).state;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_namespaceURI_8861726f144543cb = function(arg0, arg1) {
        var ret = getObject(arg1).namespaceURI;
        var ptr0 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    };
    imports.wbg.__wbg_getAttributeNames_dcff4680bfe2a224 = function(arg0) {
        var ret = getObject(arg0).getAttributeNames();
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_forEach_c2bdf17c74693f8d = function(arg0, arg1, arg2) {
        try {
            var state0 = {a: arg1, b: arg2};
            var cb0 = (arg0, arg1, arg2) => {
                const a = state0.a;
                state0.a = 0;
                try {
                    return __wbg_adapter_88(a, state0.b, arg0, arg1, arg2);
                } finally {
                    state0.a = a;
                }
            };
            getObject(arg0).forEach(cb0);
        } finally {
            state0.a = state0.b = 0;
        }
    };
    imports.wbg.__wbg_childNodes_3f5b72c829969a52 = function(arg0) {
        var ret = getObject(arg0).childNodes;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_length_7cce075f7cd84a12 = function(arg0) {
        var ret = getObject(arg0).length;
        return ret;
    };
    imports.wbg.__wbg_get_23f2600aa4163fc7 = function(arg0, arg1) {
        var ret = getObject(arg0)[arg1 >>> 0];
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    };
    imports.wbg.__wbg_nodeType_c006cd662f33b069 = function(arg0) {
        var ret = getObject(arg0).nodeType;
        return ret;
    };
    imports.wbg.__wbg_error_009e67eab9c16665 = function(arg0) {
        console.error(getObject(arg0));
    };
    imports.wbg.__wbg_textContent_230a1b70430c9a87 = function(arg0, arg1) {
        var ret = getObject(arg1).textContent;
        var ptr0 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    };
    imports.wbg.__wbg_new_147122b15d293e6e = function() { return handleError(function () {
        var ret = new AbortController();
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_new_e49c3a274bc45d18 = function() { return handleError(function () {
        var ret = new Headers();
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_append_7734c0da36ddd25a = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).append(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
    }, arguments) };
    imports.wbg.__wbg_signal_5c5a9fa291d177d7 = function(arg0) {
        var ret = getObject(arg0).signal;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_newwithstrandinit_99b3f2fe783c1e36 = function() { return handleError(function (arg0, arg1, arg2) {
        var ret = new Request(getStringFromWasm0(arg0, arg1), getObject(arg2));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_fetch_825f0bc35b153806 = function(arg0, arg1) {
        var ret = getObject(arg0).fetch(getObject(arg1));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_statusText_6994a76bf6b2cba7 = function(arg0, arg1) {
        var ret = getObject(arg1).statusText;
        var ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    };
    imports.wbg.__wbg_status_4dd3a1fab1979d66 = function(arg0) {
        var ret = getObject(arg0).status;
        return ret;
    };
    imports.wbg.__wbg_text_3ccbde6db7bfd885 = function() { return handleError(function (arg0) {
        var ret = getObject(arg0).text();
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_request_57e079697be4ba40 = function(arg0, arg1) {
        var ret = getObject(arg0).request(takeObject(arg1));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_clearLayers_7f73fd0e6343349a = function(arg0) {
        getObject(arg0).clearLayers();
    };
    imports.wbg.__wbg_newwithoptions_be4137bcd17e5f83 = function(arg0, arg1) {
        var ret = new L.Rectangle(getObject(arg0), getObject(arg1));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_addLayer_397b702f0d64e45a = function(arg0, arg1) {
        getObject(arg0).addLayer(getObject(arg1));
    };
    imports.wbg.__wbg_new_6565cc2df1daab2d = function(arg0, arg1) {
        var ret = new L.LatLng(arg0, arg1);
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_newwithoptions_a2b0067fc22204ac = function(arg0, arg1, arg2) {
        var v0 = getArrayJsValueFromWasm0(arg0, arg1).slice();
        wasm.__wbindgen_free(arg0, arg1 * 4);
        var ret = new L.Polyline(v0, getObject(arg2));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_newwithoptions_287b859e3619be1b = function(arg0, arg1) {
        var ret = new L.Circle(getObject(arg0), getObject(arg1));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_new_66c40f3edcafeaa2 = function(arg0) {
        var ret = new L.Marker(getObject(arg0));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_new_fb11c4655cdca17a = function(arg0, arg1) {
        var ret = new L.LatLngBounds(getObject(arg0), getObject(arg1));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_now_d89e184b3e3c06ec = function() {
        var ret = Date.now();
        return ret;
    };
    imports.wbg.__wbg_setView_eb0699df371cc9c9 = function(arg0, arg1, arg2) {
        getObject(arg0).setView(getObject(arg1), arg2);
    };
    imports.wbg.__wbg_invalidateSize_85a22ed632ef1fca = function(arg0, arg1) {
        getObject(arg0).invalidateSize(arg1 !== 0);
    };
    imports.wbg.__wbg_release_49e5166e5fdfa6c8 = function(arg0) {
        var ret = getObject(arg0).release();
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_setItem_8f355633f419c924 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).setItem(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
    }, arguments) };
    imports.wbg.__wbg_panTo_85aabe9534d4789c = function(arg0, arg1) {
        getObject(arg0).panTo(getObject(arg1));
    };
    imports.wbg.__wbg_wakeLock_c590e6c3835fae0a = function(arg0) {
        var ret = getObject(arg0).wakeLock;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_classList_a8c988855dad3173 = function(arg0) {
        var ret = getObject(arg0).classList;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_contains_74ffa993fdf66822 = function(arg0, arg1, arg2) {
        var ret = getObject(arg0).contains(getStringFromWasm0(arg1, arg2));
        return ret;
    };
    imports.wbg.__wbg_add_c89a42a061d74bb3 = function() { return handleError(function (arg0, arg1, arg2) {
        getObject(arg0).add(getStringFromWasm0(arg1, arg2));
    }, arguments) };
    imports.wbg.__wbg_remove_69756fa97de2df8e = function() { return handleError(function (arg0, arg1, arg2) {
        getObject(arg0).remove(getStringFromWasm0(arg1, arg2));
    }, arguments) };
    imports.wbg.__wbg_new_59cb74e423758ede = function() {
        var ret = new Error();
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_stack_558ba5917b466edd = function(arg0, arg1) {
        var ret = getObject(arg1).stack;
        var ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    };
    imports.wbg.__wbg_error_4bb6c2a97407129a = function(arg0, arg1) {
        try {
            console.error(getStringFromWasm0(arg0, arg1));
        } finally {
            wasm.__wbindgen_free(arg0, arg1);
        }
    };
    imports.wbg.__wbg_randomFillSync_64cc7d048f228ca8 = function() { return handleError(function (arg0, arg1, arg2) {
        getObject(arg0).randomFillSync(getArrayU8FromWasm0(arg1, arg2));
    }, arguments) };
    imports.wbg.__wbg_subarray_466613921b2fc6db = function(arg0, arg1, arg2) {
        var ret = getObject(arg0).subarray(arg1 >>> 0, arg2 >>> 0);
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_getRandomValues_98117e9a7e993920 = function() { return handleError(function (arg0, arg1) {
        getObject(arg0).getRandomValues(getObject(arg1));
    }, arguments) };
    imports.wbg.__wbg_length_68e13e7bbd918464 = function(arg0) {
        var ret = getObject(arg0).length;
        return ret;
    };
    imports.wbg.__wbindgen_memory = function() {
        var ret = wasm.memory;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_buffer_79a3294266d4e783 = function(arg0) {
        var ret = getObject(arg0).buffer;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_new_945397fb09fec0b8 = function(arg0) {
        var ret = new Uint8Array(getObject(arg0));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_set_223873223acf6d07 = function(arg0, arg1, arg2) {
        getObject(arg0).set(getObject(arg1), arg2 >>> 0);
    };
    imports.wbg.__wbg_process_2f24d6544ea7b200 = function(arg0) {
        var ret = getObject(arg0).process;
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_is_object = function(arg0) {
        const val = getObject(arg0);
        var ret = typeof(val) === 'object' && val !== null;
        return ret;
    };
    imports.wbg.__wbg_versions_6164651e75405d4a = function(arg0) {
        var ret = getObject(arg0).versions;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_node_4b517d861cbcb3bc = function(arg0) {
        var ret = getObject(arg0).node;
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_is_string = function(arg0) {
        var ret = typeof(getObject(arg0)) === 'string';
        return ret;
    };
    imports.wbg.__wbg_crypto_98fc271021c7d2ad = function(arg0) {
        var ret = getObject(arg0).crypto;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_msCrypto_a2cdb043d2bfe57f = function(arg0) {
        var ret = getObject(arg0).msCrypto;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_newwithlength_b7722b5594f1dc21 = function(arg0) {
        var ret = new Uint8Array(arg0 >>> 0);
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_modulerequire_3440a4bcf44437db = function() { return handleError(function (arg0, arg1) {
        var ret = module.require(getStringFromWasm0(arg0, arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_setTimeout_56c7fbf887ce45de = function() { return handleError(function (arg0, arg1, arg2) {
        var ret = getObject(arg0).setTimeout(getObject(arg1), arg2);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_setTimeout_7c4b174605b75e39 = function() { return handleError(function (arg0, arg1, arg2) {
        var ret = getObject(arg0).setTimeout(getObject(arg1), arg2);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_Window_f826a1dec163bacb = function(arg0) {
        var ret = getObject(arg0).Window;
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_is_undefined = function(arg0) {
        var ret = getObject(arg0) === undefined;
        return ret;
    };
    imports.wbg.__wbg_WorkerGlobalScope_967d186155183d38 = function(arg0) {
        var ret = getObject(arg0).WorkerGlobalScope;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_clearTimeout_60252f615c569bcf = function(arg0, arg1) {
        getObject(arg0).clearTimeout(arg1);
    };
    imports.wbg.__wbg_clearTimeout_314f616ced084484 = function(arg0, arg1) {
        getObject(arg0).clearTimeout(arg1);
    };
    imports.wbg.__wbg_length_c5fa152b8c3f311f = function(arg0) {
        var ret = getObject(arg0).length;
        return ret;
    };
    imports.wbg.__wbg_get_73c087db0a496c21 = function(arg0, arg1) {
        var ret = getObject(arg0)[arg1 >>> 0];
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_self_b4546ea7b590539e = function() { return handleError(function () {
        var ret = self.self;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_window_c279fea81f426a68 = function() { return handleError(function () {
        var ret = window.window;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_globalThis_038a6ea0ff17789f = function() { return handleError(function () {
        var ret = globalThis.globalThis;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_global_4f93ce884bcee597 = function() { return handleError(function () {
        var ret = global.global;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_newnoargs_1a11e7e8c906996c = function(arg0, arg1) {
        var ret = new Function(getStringFromWasm0(arg0, arg1));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_call_e91f71ddf1f45cff = function() { return handleError(function (arg0, arg1) {
        var ret = getObject(arg0).call(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_set_d29a397c9cc5d746 = function() { return handleError(function (arg0, arg1, arg2) {
        var ret = Reflect.set(getObject(arg0), getObject(arg1), getObject(arg2));
        return ret;
    }, arguments) };
    imports.wbg.__wbg_decodeURIComponent_7466c397b7b27b8c = function() { return handleError(function (arg0, arg1) {
        var ret = decodeURIComponent(getStringFromWasm0(arg0, arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_encodeURIComponent_3bb6f911d5cd19cb = function(arg0, arg1) {
        var ret = encodeURIComponent(getStringFromWasm0(arg0, arg1));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_abort_7b87cd42b53665b4 = function(arg0) {
        getObject(arg0).abort();
    };
    imports.wbg.__wbg_instanceof_Window_fac4f1f8e3c61c1f = function(arg0) {
        var ret = getObject(arg0) instanceof Window;
        return ret;
    };
    imports.wbg.__wbg_createTextNode_6a2584b3a9fa0d19 = function(arg0, arg1, arg2) {
        var ret = getObject(arg0).createTextNode(getStringFromWasm0(arg1, arg2));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_instanceof_Node_ee447f4cc6d529e5 = function(arg0) {
        var ret = getObject(arg0) instanceof Node;
        return ret;
    };
    imports.wbg.__wbg_insertBefore_ea947b64ef055476 = function() { return handleError(function (arg0, arg1, arg2) {
        var ret = getObject(arg0).insertBefore(getObject(arg1), getObject(arg2));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_replaceChild_59348bdcca4bc5f1 = function() { return handleError(function (arg0, arg1, arg2) {
        var ret = getObject(arg0).replaceChild(getObject(arg1), getObject(arg2));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_location_27d553bf206d4861 = function(arg0) {
        var ret = getObject(arg0).location;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_href_44cae7c5674402e1 = function() { return handleError(function (arg0, arg1) {
        var ret = getObject(arg1).href;
        var ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    }, arguments) };
    imports.wbg.__wbg_searchParams_76da5cd7cb92b763 = function(arg0) {
        var ret = getObject(arg0).searchParams;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_from_28631399e1e647cb = function(arg0) {
        var ret = Array.from(getObject(arg0));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_setvalue_9bba6347da492e7b = function(arg0, arg1, arg2) {
        getObject(arg0).value = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_setvalue_346cd661760e3b2e = function(arg0, arg1) {
        getObject(arg0).value = arg1;
    };
    imports.wbg.__wbg_setvalue_bff7ed2fa0f3da24 = function(arg0, arg1, arg2) {
        getObject(arg0).value = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_setvalue_c3ed819a3212091e = function(arg0, arg1, arg2) {
        getObject(arg0).value = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_setvalue_38c0548eef796751 = function(arg0, arg1, arg2) {
        getObject(arg0).value = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_setvalue_e6f44588bd4707bc = function(arg0, arg1) {
        getObject(arg0).value = arg1;
    };
    imports.wbg.__wbg_setvalue_d11db6de8b2947b8 = function(arg0, arg1) {
        getObject(arg0).value = arg1;
    };
    imports.wbg.__wbg_setvalue_67edc0d66812a510 = function(arg0, arg1, arg2) {
        getObject(arg0).value = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_setvalue_58b08a1274906d93 = function(arg0, arg1, arg2) {
        getObject(arg0).value = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_activeElement_017f774ea9395462 = function(arg0) {
        var ret = getObject(arg0).activeElement;
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    };
    imports.wbg.__wbg_is_50b1fea1dc5b6e28 = function(arg0, arg1) {
        var ret = Object.is(getObject(arg0), getObject(arg1));
        return ret;
    };
    imports.wbg.__wbg_setvalue_53c8ebe1d61c347a = function(arg0, arg1, arg2) {
        getObject(arg0).value = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_selectionStart_6e5f3c2ade8c4bf5 = function() { return handleError(function (arg0, arg1) {
        var ret = getObject(arg1).selectionStart;
        getInt32Memory0()[arg0 / 4 + 1] = isLikeNone(ret) ? 0 : ret;
        getInt32Memory0()[arg0 / 4 + 0] = !isLikeNone(ret);
    }, arguments) };
    imports.wbg.__wbg_selectionEnd_50ffa55a4a56ae9d = function() { return handleError(function (arg0, arg1) {
        var ret = getObject(arg1).selectionEnd;
        getInt32Memory0()[arg0 / 4 + 1] = isLikeNone(ret) ? 0 : ret;
        getInt32Memory0()[arg0 / 4 + 0] = !isLikeNone(ret);
    }, arguments) };
    imports.wbg.__wbg_setselectionStart_345c4ff47c675dfa = function() { return handleError(function (arg0, arg1, arg2) {
        getObject(arg0).selectionStart = arg1 === 0 ? undefined : arg2 >>> 0;
    }, arguments) };
    imports.wbg.__wbg_setselectionEnd_2d06daca5815f166 = function() { return handleError(function (arg0, arg1, arg2) {
        getObject(arg0).selectionEnd = arg1 === 0 ? undefined : arg2 >>> 0;
    }, arguments) };
    imports.wbg.__wbg_instanceof_HtmlMenuItemElement_aa1560cf8979598c = function(arg0) {
        var ret = getObject(arg0) instanceof HTMLMenuItemElement;
        return ret;
    };
    imports.wbg.__wbg_setchecked_b62bcafd4b6be143 = function(arg0, arg1) {
        getObject(arg0).checked = arg1 !== 0;
    };
    imports.wbg.__wbg_setchecked_1fdd605f56c99f90 = function(arg0, arg1) {
        getObject(arg0).checked = arg1 !== 0;
    };
    imports.wbg.__wbg_localStorage_f35f647a18f9add7 = function() { return handleError(function (arg0) {
        var ret = getObject(arg0).localStorage;
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_string_get = function(arg0, arg1) {
        const obj = getObject(arg1);
        var ret = typeof(obj) === 'string' ? obj : undefined;
        var ptr0 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    };
    imports.wbg.__wbindgen_debug_string = function(arg0, arg1) {
        var ret = debugString(getObject(arg1));
        var ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    };
    imports.wbg.__wbindgen_throw = function(arg0, arg1) {
        throw new Error(getStringFromWasm0(arg0, arg1));
    };
    imports.wbg.__wbg_then_6d5072fec3fdb237 = function(arg0, arg1) {
        var ret = getObject(arg0).then(getObject(arg1));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_then_4f3c7f6f3d36634a = function(arg0, arg1, arg2) {
        var ret = getObject(arg0).then(getObject(arg1), getObject(arg2));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_resolve_7161ec6fd5b1cd29 = function(arg0) {
        var ret = Promise.resolve(getObject(arg0));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_createElement_8a4eea3a05d8804d = function() { return handleError(function (arg0, arg1, arg2) {
        var ret = getObject(arg0).createElement(getStringFromWasm0(arg1, arg2));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_querySelector_ff7871012dacd8ad = function() { return handleError(function (arg0, arg1, arg2) {
        var ret = getObject(arg0).querySelector(getStringFromWasm0(arg1, arg2));
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_tagName_70904fd4db183fec = function(arg0, arg1) {
        var ret = getObject(arg1).tagName;
        var ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    };
    imports.wbg.__wbg_getAttribute_906531fdd693b7f2 = function(arg0, arg1, arg2, arg3) {
        var ret = getObject(arg1).getAttribute(getStringFromWasm0(arg2, arg3));
        var ptr0 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    };
    imports.wbg.__wbg_removeAttribute_f579d1395b5a1244 = function() { return handleError(function (arg0, arg1, arg2) {
        getObject(arg0).removeAttribute(getStringFromWasm0(arg1, arg2));
    }, arguments) };
    imports.wbg.__wbg_setAttribute_936a229ff5ccc5c9 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
        getObject(arg0).setAttribute(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
    }, arguments) };
    imports.wbg.__wbg_addEventListener_de2304ba2cd4ae89 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        getObject(arg0).addEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3));
    }, arguments) };
    imports.wbg.__wbg_type_89134f0ae8882b9c = function(arg0, arg1) {
        var ret = getObject(arg1).type;
        var ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    };
    imports.wbg.__wbg_appendChild_f52907370cfda116 = function() { return handleError(function (arg0, arg1) {
        var ret = getObject(arg0).appendChild(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_removeChild_a5b181d3ef9e42e8 = function() { return handleError(function (arg0, arg1) {
        var ret = getObject(arg0).removeChild(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_pathname_f017b89f4454ebd9 = function(arg0, arg1) {
        var ret = getObject(arg1).pathname;
        var ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    };
    imports.wbg.__wbg_hash_0d6dc2bc6d5da7ff = function(arg0, arg1) {
        var ret = getObject(arg1).hash;
        var ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        getInt32Memory0()[arg0 / 4 + 1] = len0;
        getInt32Memory0()[arg0 / 4 + 0] = ptr0;
    };
    imports.wbg.__wbg_newwithbase_f553b0cf67715216 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        var ret = new URL(getStringFromWasm0(arg0, arg1), getStringFromWasm0(arg2, arg3));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_closure_wrapper257 = function(arg0, arg1, arg2) {
        var ret = makeMutClosure(arg0, arg1, 10, __wbg_adapter_30);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper259 = function(arg0, arg1, arg2) {
        var ret = makeMutClosure(arg0, arg1, 14, __wbg_adapter_33);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper261 = function(arg0, arg1, arg2) {
        var ret = makeMutClosure(arg0, arg1, 18, __wbg_adapter_36);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper376 = function(arg0, arg1, arg2) {
        var ret = makeMutClosure(arg0, arg1, 46, __wbg_adapter_39);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper576 = function(arg0, arg1, arg2) {
        var ret = makeMutClosure(arg0, arg1, 80, __wbg_adapter_42);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper589 = function(arg0, arg1, arg2) {
        var ret = makeMutClosure(arg0, arg1, 49, __wbg_adapter_45);
        return addHeapObject(ret);
    };

    if (typeof input === 'string' || (typeof Request === 'function' && input instanceof Request) || (typeof URL === 'function' && input instanceof URL)) {
        input = fetch(input);
    }



    const { instance, module } = await load(await input, imports);

    wasm = instance.exports;
    init.__wbindgen_wasm_module = module;
    wasm.__wbindgen_start();
    return wasm;
}

export default init;

