"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports["default"] = exports.ContainerService = void 0;

function _classCallCheck(instance, Constructor) { if (!(instance instanceof Constructor)) { throw new TypeError("Cannot call a class as a function"); } }

function _defineProperties(target, props) { for (var i = 0; i < props.length; i++) { var descriptor = props[i]; descriptor.enumerable = descriptor.enumerable || false; descriptor.configurable = true; if ("value" in descriptor) descriptor.writable = true; Object.defineProperty(target, descriptor.key, descriptor); } }

function _createClass(Constructor, protoProps, staticProps) { if (protoProps) _defineProperties(Constructor.prototype, protoProps); if (staticProps) _defineProperties(Constructor, staticProps); return Constructor; }

function _typeof(obj) { if (typeof Symbol === "function" && typeof Symbol.iterator === "symbol") { _typeof = function _typeof(obj) { return typeof obj; }; } else { _typeof = function _typeof(obj) { return obj && typeof Symbol === "function" && obj.constructor === Symbol && obj !== Symbol.prototype ? "symbol" : typeof obj; }; } return _typeof(obj); }

var importMeta = {
  url: new URL('/stremio_core_web.js', document.baseURI).href
};
var wasm;
var heap = new Array(32);
heap.fill(undefined);
heap.push(undefined, null, true, false);
var heap_next = heap.length;

function addHeapObject(obj) {
  if (heap_next === heap.length) heap.push(heap.length + 1);
  var idx = heap_next;
  heap_next = heap[idx];
  heap[idx] = obj;
  return idx;
}

function __wbg_elem_binding0(arg0, arg1, arg2) {
  wasm.__wbg_function_table.get(103)(arg0, arg1, addHeapObject(arg2));
}

function __wbg_elem_binding1(arg0, arg1, arg2, arg3) {
  wasm.__wbg_function_table.get(58)(arg0, arg1, addHeapObject(arg2), addHeapObject(arg3));
}

var stack_pointer = 32;

function addBorrowedObject(obj) {
  if (stack_pointer == 1) throw new Error('out of js stack');
  heap[--stack_pointer] = obj;
  return stack_pointer;
}

function getObject(idx) {
  return heap[idx];
}

function dropObject(idx) {
  if (idx < 36) return;
  heap[idx] = heap_next;
  heap_next = idx;
}

function takeObject(idx) {
  var ret = getObject(idx);
  dropObject(idx);
  return ret;
}

var WASM_VECTOR_LEN = 0;
var cachedTextEncoder = new TextEncoder('utf-8');
var cachegetUint8Memory = null;

function getUint8Memory() {
  if (cachegetUint8Memory === null || cachegetUint8Memory.buffer !== wasm.memory.buffer) {
    cachegetUint8Memory = new Uint8Array(wasm.memory.buffer);
  }

  return cachegetUint8Memory;
}

var passStringToWasm;

if (typeof cachedTextEncoder.encodeInto === 'function') {
  passStringToWasm = function passStringToWasm(arg) {
    var size = arg.length;

    var ptr = wasm.__wbindgen_malloc(size);

    var offset = 0;
    {
      var mem = getUint8Memory();

      for (; offset < arg.length; offset++) {
        var code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
      }
    }

    if (offset !== arg.length) {
      arg = arg.slice(offset);
      ptr = wasm.__wbindgen_realloc(ptr, size, size = offset + arg.length * 3);
      var view = getUint8Memory().subarray(ptr + offset, ptr + size);
      var ret = cachedTextEncoder.encodeInto(arg, view);
      offset += ret.written;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
  };
} else {
  passStringToWasm = function passStringToWasm(arg) {
    var size = arg.length;

    var ptr = wasm.__wbindgen_malloc(size);

    var offset = 0;
    {
      var mem = getUint8Memory();

      for (; offset < arg.length; offset++) {
        var code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
      }
    }

    if (offset !== arg.length) {
      var buf = cachedTextEncoder.encode(arg.slice(offset));
      ptr = wasm.__wbindgen_realloc(ptr, size, size = offset + buf.length);
      getUint8Memory().set(buf, ptr + offset);
      offset += buf.length;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
  };
}

var cachegetUint32Memory = null;

function getUint32Memory() {
  if (cachegetUint32Memory === null || cachegetUint32Memory.buffer !== wasm.memory.buffer) {
    cachegetUint32Memory = new Uint32Array(wasm.memory.buffer);
  }

  return cachegetUint32Memory;
}

function handleError(e) {
  wasm.__wbindgen_exn_store(addHeapObject(e));
}

function isLikeNone(x) {
  return x === undefined || x === null;
}

var cachedTextDecoder = new TextDecoder('utf-8');

function getStringFromWasm(ptr, len) {
  return cachedTextDecoder.decode(getUint8Memory().subarray(ptr, ptr + len));
}

var cachegetInt32Memory = null;

function getInt32Memory() {
  if (cachegetInt32Memory === null || cachegetInt32Memory.buffer !== wasm.memory.buffer) {
    cachegetInt32Memory = new Int32Array(wasm.memory.buffer);
  }

  return cachegetInt32Memory;
}

function debugString(val) {
  // primitive types
  var type = _typeof(val);

  if (type == 'number' || type == 'boolean' || val == null) {
    return "".concat(val);
  }

  if (type == 'string') {
    return "\"".concat(val, "\"");
  }

  if (type == 'symbol') {
    var description = val.description;

    if (description == null) {
      return 'Symbol';
    } else {
      return "Symbol(".concat(description, ")");
    }
  }

  if (type == 'function') {
    var name = val.name;

    if (typeof name == 'string' && name.length > 0) {
      return "Function(".concat(name, ")");
    } else {
      return 'Function';
    }
  } // objects


  if (Array.isArray(val)) {
    var length = val.length;
    var debug = '[';

    if (length > 0) {
      debug += debugString(val[0]);
    }

    for (var i = 1; i < length; i++) {
      debug += ', ' + debugString(val[i]);
    }

    debug += ']';
    return debug;
  } // Test for built-in


  var builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
  var className;

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
  } // errors


  if (val instanceof Error) {
    return "".concat(val.name, ": ").concat(val.message, "\n").concat(val.stack);
  } // TODO we could test for more things here, like `Set`s and `Map`s.


  return className;
}
/**
*/


var ContainerService =
/*#__PURE__*/
function () {
  _createClass(ContainerService, [{
    key: "free",
    value: function free() {
      var ptr = this.ptr;
      this.ptr = 0;

      wasm.__wbg_containerservice_free(ptr);
    }
    /**
    * @param {any} emit
    * @returns {ContainerService}
    */

  }], [{
    key: "__wrap",
    value: function __wrap(ptr) {
      var obj = Object.create(ContainerService.prototype);
      obj.ptr = ptr;
      return obj;
    }
  }]);

  function ContainerService(emit) {
    _classCallCheck(this, ContainerService);

    var ret = wasm.containerservice_new(addHeapObject(emit));
    return ContainerService.__wrap(ret);
  }
  /**
  * @param {any} action_js
  */


  _createClass(ContainerService, [{
    key: "dispatch",
    value: function dispatch(action_js) {
      try {
        wasm.containerservice_dispatch(this.ptr, addBorrowedObject(action_js));
      } finally {
        heap[stack_pointer++] = undefined;
      }
    }
    /**
    * @returns {any}
    */

  }, {
    key: "get_state",
    value: function get_state() {
      var ret = wasm.containerservice_get_state(this.ptr);
      return takeObject(ret);
    }
  }]);

  return ContainerService;
}();

exports.ContainerService = ContainerService;

function init(module) {
  if (typeof module === 'undefined') {
    module = importMeta.url.replace(/\.js$/, '_bg.wasm');
  }

  var result;
  var imports = {};
  imports.wbg = {};

  imports.wbg.__wbindgen_object_drop_ref = function (arg0) {
    takeObject(arg0);
  };

  imports.wbg.__wbg_instanceof_Error_3c415a41cc2164b9 = function (arg0) {
    var ret = getObject(arg0) instanceof Error;
    return ret;
  };

  imports.wbg.__wbg_toString_fac1793137277b7f = function (arg0) {
    var ret = getObject(arg0).toString();
    return addHeapObject(ret);
  };

  imports.wbg.__wbindgen_string_get = function (arg0, arg1) {
    var obj = getObject(arg0);
    if (typeof obj !== 'string') return 0;
    var ptr = passStringToWasm(obj);
    getUint32Memory()[arg1 / 4] = WASM_VECTOR_LEN;
    var ret = ptr;
    return ret;
  };

  imports.wbg.__widl_f_local_storage_Window = function (arg0) {
    try {
      var ret = getObject(arg0).localStorage;
      return isLikeNone(ret) ? 0 : addHeapObject(ret);
    } catch (e) {
      handleError(e);
    }
  };

  imports.wbg.__wbg_new_1719c88e1a2035ea = function (arg0, arg1) {
    var state0 = {
      a: arg0,
      b: arg1
    };

    var cb0 = function cb0(arg0, arg1) {
      var a = state0.a;
      state0.a = 0;

      try {
        return __wbg_elem_binding1(a, state0.b, arg0, arg1);
      } finally {
        state0.a = a;
      }
    };

    try {
      var ret = new Promise(cb0);
      return addHeapObject(ret);
    } finally {
      state0.a = state0.b = 0;
    }
  };

  imports.wbg.__wbg_globalThis_36c1f2e85948e420 = function () {
    try {
      var ret = globalThis.globalThis;
      return addHeapObject(ret);
    } catch (e) {
      handleError(e);
    }
  };

  imports.wbg.__wbg_self_73c7a601ff857345 = function () {
    try {
      var ret = self.self;
      return addHeapObject(ret);
    } catch (e) {
      handleError(e);
    }
  };

  imports.wbg.__wbg_window_ca735e04cb2b0566 = function () {
    try {
      var ret = window.window;
      return addHeapObject(ret);
    } catch (e) {
      handleError(e);
    }
  };

  imports.wbg.__wbg_global_99312a595fd2e761 = function () {
    try {
      var ret = global.global;
      return addHeapObject(ret);
    } catch (e) {
      handleError(e);
    }
  };

  imports.wbg.__wbindgen_is_undefined = function (arg0) {
    var ret = getObject(arg0) === undefined;
    return ret;
  };

  imports.wbg.__wbg_newnoargs_6ad69a50998c5acb = function (arg0, arg1) {
    var ret = new Function(getStringFromWasm(arg0, arg1));
    return addHeapObject(ret);
  };

  imports.wbg.__wbg_call_4499dca0c553c196 = function (arg0, arg1) {
    try {
      var ret = getObject(arg0).call(getObject(arg1));
      return addHeapObject(ret);
    } catch (e) {
      handleError(e);
    }
  };

  imports.wbg.__wbg_call_fdde574e8abf6327 = function (arg0, arg1, arg2) {
    try {
      var ret = getObject(arg0).call(getObject(arg1), getObject(arg2));
      return addHeapObject(ret);
    } catch (e) {
      handleError(e);
    }
  };

  imports.wbg.__wbg_set_0718caf2a62a5c4f = function (arg0, arg1, arg2) {
    try {
      var ret = Reflect.set(getObject(arg0), getObject(arg1), getObject(arg2));
      return ret;
    } catch (e) {
      handleError(e);
    }
  };

  imports.wbg.__wbg_new_2d6a830207834e5d = function () {
    var ret = new Object();
    return addHeapObject(ret);
  };

  imports.wbg.__wbindgen_string_new = function (arg0, arg1) {
    var ret = getStringFromWasm(arg0, arg1);
    return addHeapObject(ret);
  };

  imports.wbg.__widl_f_fetch_with_request_Window = function (arg0, arg1) {
    var ret = getObject(arg0).fetch(getObject(arg1));
    return addHeapObject(ret);
  };

  imports.wbg.__widl_instanceof_Response = function (arg0) {
    var ret = getObject(arg0) instanceof Response;
    return ret;
  };

  imports.wbg.__widl_f_status_Response = function (arg0) {
    var ret = getObject(arg0).status;
    return ret;
  };

  imports.wbg.__wbindgen_json_serialize = function (arg0, arg1) {
    var ret = JSON.stringify(getObject(arg1));
    var ret0 = passStringToWasm(ret);
    var ret1 = WASM_VECTOR_LEN;
    getInt32Memory()[arg0 / 4 + 0] = ret0;
    getInt32Memory()[arg0 / 4 + 1] = ret1;
  };

  imports.wbg.__wbindgen_json_parse = function (arg0, arg1) {
    var ret = JSON.parse(getStringFromWasm(arg0, arg1));
    return addHeapObject(ret);
  };

  imports.wbg.__wbindgen_debug_string = function (arg0, arg1) {
    var ret = debugString(getObject(arg1));
    var ret0 = passStringToWasm(ret);
    var ret1 = WASM_VECTOR_LEN;
    getInt32Memory()[arg0 / 4 + 0] = ret0;
    getInt32Memory()[arg0 / 4 + 1] = ret1;
  };

  imports.wbg.__wbindgen_throw = function (arg0, arg1) {
    throw new Error(getStringFromWasm(arg0, arg1));
  };

  imports.wbg.__wbindgen_cb_drop = function (arg0) {
    var obj = takeObject(arg0).original;

    if (obj.cnt-- == 1) {
      obj.a = 0;
      return true;
    }

    var ret = false;
    return ret;
  };

  imports.wbg.__wbg_then_0fe88013efbd2711 = function (arg0, arg1, arg2) {
    var ret = getObject(arg0).then(getObject(arg1), getObject(arg2));
    return addHeapObject(ret);
  };

  imports.wbg.__wbg_resolve_bacd3bf49c19a0f8 = function (arg0) {
    var ret = Promise.resolve(getObject(arg0));
    return addHeapObject(ret);
  };

  imports.wbg.__wbg_then_3d6c84182dd53850 = function (arg0, arg1) {
    var ret = getObject(arg0).then(getObject(arg1));
    return addHeapObject(ret);
  };

  imports.wbg.__wbindgen_object_clone_ref = function (arg0) {
    var ret = getObject(arg0);
    return addHeapObject(ret);
  };

  imports.wbg.__widl_instanceof_Window = function (arg0) {
    var ret = getObject(arg0) instanceof Window;
    return ret;
  };

  imports.wbg.__widl_f_new_with_str_and_init_Request = function (arg0, arg1, arg2) {
    try {
      var ret = new Request(getStringFromWasm(arg0, arg1), getObject(arg2));
      return addHeapObject(ret);
    } catch (e) {
      handleError(e);
    }
  };

  imports.wbg.__widl_f_json_Response = function (arg0) {
    try {
      var ret = getObject(arg0).json();
      return addHeapObject(ret);
    } catch (e) {
      handleError(e);
    }
  };

  imports.wbg.__widl_f_get_item_Storage = function (arg0, arg1, arg2, arg3) {
    try {
      var ret = getObject(arg1).getItem(getStringFromWasm(arg2, arg3));
      var ptr0 = isLikeNone(ret) ? 0 : passStringToWasm(ret);
      var len0 = WASM_VECTOR_LEN;
      var ret0 = ptr0;
      var ret1 = len0;
      getInt32Memory()[arg0 / 4 + 0] = ret0;
      getInt32Memory()[arg0 / 4 + 1] = ret1;
    } catch (e) {
      handleError(e);
    }
  };

  imports.wbg.__widl_f_remove_item_Storage = function (arg0, arg1, arg2) {
    try {
      getObject(arg0).removeItem(getStringFromWasm(arg1, arg2));
    } catch (e) {
      handleError(e);
    }
  };

  imports.wbg.__widl_f_set_item_Storage = function (arg0, arg1, arg2, arg3, arg4) {
    try {
      getObject(arg0).setItem(getStringFromWasm(arg1, arg2), getStringFromWasm(arg3, arg4));
    } catch (e) {
      handleError(e);
    }
  };

  imports.wbg.__wbindgen_closure_wrapper2913 = function (arg0, arg1, arg2) {
    var state = {
      a: arg0,
      b: arg1,
      cnt: 1
    };

    var real = function real(arg0) {
      state.cnt++;
      var a = state.a;
      state.a = 0;

      try {
        return __wbg_elem_binding0(a, state.b, arg0);
      } finally {
        if (--state.cnt === 0) wasm.__wbg_function_table.get(104)(a, state.b);else state.a = a;
      }
    };

    real.original = state;
    var ret = real;
    return addHeapObject(ret);
  };

  if (typeof URL === 'function' && module instanceof URL || typeof module === 'string' || typeof Request === 'function' && module instanceof Request) {
    var response = fetch(module);

    if (typeof WebAssembly.instantiateStreaming === 'function') {
      result = WebAssembly.instantiateStreaming(response, imports)["catch"](function (e) {
        return response.then(function (r) {
          if (r.headers.get('Content-Type') != 'application/wasm') {
            console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);
            return r.arrayBuffer();
          } else {
            throw e;
          }
        }).then(function (bytes) {
          return WebAssembly.instantiate(bytes, imports);
        });
      });
    } else {
      result = response.then(function (r) {
        return r.arrayBuffer();
      }).then(function (bytes) {
        return WebAssembly.instantiate(bytes, imports);
      });
    }
  } else {
    result = WebAssembly.instantiate(module, imports).then(function (result) {
      if (result instanceof WebAssembly.Instance) {
        return {
          instance: result,
          module: module
        };
      } else {
        return result;
      }
    });
  }

  return result.then(function (_ref) {
    var instance = _ref.instance,
        module = _ref.module;
    wasm = instance.exports;
    init.__wbindgen_wasm_module = module;
    return wasm;
  });
}

var _default = init;
exports["default"] = _default;
