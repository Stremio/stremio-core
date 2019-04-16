"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports["default"] = exports.ContainerService = void 0;

function _classCallCheck(instance, Constructor) { if (!(instance instanceof Constructor)) { throw new TypeError("Cannot call a class as a function"); } }

function _defineProperties(target, props) { for (var i = 0; i < props.length; i++) { var descriptor = props[i]; descriptor.enumerable = descriptor.enumerable || false; descriptor.configurable = true; if ("value" in descriptor) descriptor.writable = true; Object.defineProperty(target, descriptor.key, descriptor); } }

function _createClass(Constructor, protoProps, staticProps) { if (protoProps) _defineProperties(Constructor.prototype, protoProps); if (staticProps) _defineProperties(Constructor, staticProps); return Constructor; }

function _typeof(obj) { if (typeof Symbol === "function" && typeof Symbol.iterator === "symbol") { _typeof = function _typeof(obj) { return typeof obj; }; } else { _typeof = function _typeof(obj) { return obj && typeof Symbol === "function" && obj.constructor === Symbol && obj !== Symbol.prototype ? "symbol" : typeof obj; }; } return _typeof(obj); }

var __exports = {};
var wasm;
var cachedTextDecoder = new TextDecoder('utf-8');
var cachegetUint8Memory = null;

function getUint8Memory() {
  if (cachegetUint8Memory === null || cachegetUint8Memory.buffer !== wasm.memory.buffer) {
    cachegetUint8Memory = new Uint8Array(wasm.memory.buffer);
  }

  return cachegetUint8Memory;
}

function getStringFromWasm(ptr, len) {
  return cachedTextDecoder.decode(getUint8Memory().subarray(ptr, ptr + len));
}

var heap = new Array(32);
heap.fill(undefined);
heap.push(undefined, null, true, false);

function getObject(idx) {
  return heap[idx];
}

var heap_next = heap.length;

function addHeapObject(obj) {
  if (heap_next === heap.length) heap.push(heap.length + 1);
  var idx = heap_next;
  heap_next = heap[idx];
  heap[idx] = obj;
  return idx;
}

var cachegetUint32Memory = null;

function getUint32Memory() {
  if (cachegetUint32Memory === null || cachegetUint32Memory.buffer !== wasm.memory.buffer) {
    cachegetUint32Memory = new Uint32Array(wasm.memory.buffer);
  }

  return cachegetUint32Memory;
}

function handleError(exnptr, e) {
  var view = getUint32Memory();
  view[exnptr / 4] = 1;
  view[exnptr / 4 + 1] = addHeapObject(e);
}

function __widl_f_new_with_str_and_init_Request(arg0, arg1, arg2, exnptr) {
  var varg0 = getStringFromWasm(arg0, arg1);

  try {
    return addHeapObject(new Request(varg0, getObject(arg2)));
  } catch (e) {
    handleError(exnptr, e);
  }
}

__exports.__widl_f_new_with_str_and_init_Request = __widl_f_new_with_str_and_init_Request;

function __widl_instanceof_Response(idx) {
  return getObject(idx) instanceof Response ? 1 : 0;
}

__exports.__widl_instanceof_Response = __widl_instanceof_Response;

function __widl_f_json_Response(arg0, exnptr) {
  try {
    return addHeapObject(getObject(arg0).json());
  } catch (e) {
    handleError(exnptr, e);
  }
}

__exports.__widl_f_json_Response = __widl_f_json_Response;
var WASM_VECTOR_LEN = 0;
var cachedTextEncoder = new TextEncoder('utf-8');
var passStringToWasm;

if (typeof cachedTextEncoder.encodeInto === 'function') {
  passStringToWasm = function passStringToWasm(arg) {
    var size = arg.length;

    var ptr = wasm.__wbindgen_malloc(size);

    var writeOffset = 0;

    while (true) {
      var view = getUint8Memory().subarray(ptr + writeOffset, ptr + size);

      var _cachedTextEncoder$en = cachedTextEncoder.encodeInto(arg, view),
          read = _cachedTextEncoder$en.read,
          written = _cachedTextEncoder$en.written;

      writeOffset += written;

      if (read === arg.length) {
        break;
      }

      arg = arg.substring(read);
      ptr = wasm.__wbindgen_realloc(ptr, size, size += arg.length * 3);
    }

    WASM_VECTOR_LEN = writeOffset;
    return ptr;
  };
} else {
  passStringToWasm = function passStringToWasm(arg) {
    var buf = cachedTextEncoder.encode(arg);

    var ptr = wasm.__wbindgen_malloc(buf.length);

    getUint8Memory().set(buf, ptr);
    WASM_VECTOR_LEN = buf.length;
    return ptr;
  };
}

function isLikeNone(x) {
  return x === undefined || x === null;
}

function __widl_f_get_item_Storage(ret, arg0, arg1, arg2, exnptr) {
  var varg1 = getStringFromWasm(arg1, arg2);

  try {
    var val = getObject(arg0).getItem(varg1);
    var retptr = isLikeNone(val) ? [0, 0] : passStringToWasm(val);
    var retlen = WASM_VECTOR_LEN;
    var mem = getUint32Memory();
    mem[ret / 4] = retptr;
    mem[ret / 4 + 1] = retlen;
  } catch (e) {
    handleError(exnptr, e);
  }
}

__exports.__widl_f_get_item_Storage = __widl_f_get_item_Storage;

function __widl_f_set_item_Storage(arg0, arg1, arg2, arg3, arg4, exnptr) {
  var varg1 = getStringFromWasm(arg1, arg2);
  var varg3 = getStringFromWasm(arg3, arg4);

  try {
    getObject(arg0).setItem(varg1, varg3);
  } catch (e) {
    handleError(exnptr, e);
  }
}

__exports.__widl_f_set_item_Storage = __widl_f_set_item_Storage;

function __widl_instanceof_Window(idx) {
  return getObject(idx) instanceof Window ? 1 : 0;
}

__exports.__widl_instanceof_Window = __widl_instanceof_Window;

function __widl_f_local_storage_Window(arg0, exnptr) {
  try {
    var val = getObject(arg0).localStorage;
    return isLikeNone(val) ? 0 : addHeapObject(val);
  } catch (e) {
    handleError(exnptr, e);
  }
}

__exports.__widl_f_local_storage_Window = __widl_f_local_storage_Window;

function __widl_f_fetch_with_request_Window(arg0, arg1) {
  return addHeapObject(getObject(arg0).fetch(getObject(arg1)));
}

__exports.__widl_f_fetch_with_request_Window = __widl_f_fetch_with_request_Window;

function __wbg_instanceof_Error_19bd7a7890e3fd6c(idx) {
  return getObject(idx) instanceof Error ? 1 : 0;
}

__exports.__wbg_instanceof_Error_19bd7a7890e3fd6c = __wbg_instanceof_Error_19bd7a7890e3fd6c;

function __wbg_toString_51b1ec90e420207d(arg0) {
  return addHeapObject(getObject(arg0).toString());
}

__exports.__wbg_toString_51b1ec90e420207d = __wbg_toString_51b1ec90e420207d;

function __wbg_newnoargs_cb83ac9bfa714d41(arg0, arg1) {
  var varg0 = getStringFromWasm(arg0, arg1);
  return addHeapObject(new Function(varg0));
}

__exports.__wbg_newnoargs_cb83ac9bfa714d41 = __wbg_newnoargs_cb83ac9bfa714d41;

function __wbg_call_75755734bfea4d37(arg0, arg1, exnptr) {
  try {
    return addHeapObject(getObject(arg0).call(getObject(arg1)));
  } catch (e) {
    handleError(exnptr, e);
  }
}

__exports.__wbg_call_75755734bfea4d37 = __wbg_call_75755734bfea4d37;

function __wbg_call_0492299fb1f5901e(arg0, arg1, arg2, exnptr) {
  try {
    return addHeapObject(getObject(arg0).call(getObject(arg1), getObject(arg2)));
  } catch (e) {
    handleError(exnptr, e);
  }
}

__exports.__wbg_call_0492299fb1f5901e = __wbg_call_0492299fb1f5901e;

function __wbg_new_2dc379b3ba5ebef6() {
  return addHeapObject(new Object());
}

__exports.__wbg_new_2dc379b3ba5ebef6 = __wbg_new_2dc379b3ba5ebef6;

function __wbg_set_2624d1f32a3776d1(arg0, arg1, arg2, exnptr) {
  try {
    return Reflect.set(getObject(arg0), getObject(arg1), getObject(arg2));
  } catch (e) {
    handleError(exnptr, e);
  }
}

__exports.__wbg_set_2624d1f32a3776d1 = __wbg_set_2624d1f32a3776d1;

function __wbg_new_ce158cf1048d4c17(arg0, arg1) {
  var cbarg0 = function cbarg0(arg0, arg1) {
    var a = this.a;
    this.a = 0;

    try {
      return this.f(a, this.b, addHeapObject(arg0), addHeapObject(arg1));
    } finally {
      this.a = a;
    }
  };

  cbarg0.f = wasm.__wbg_function_table.get(21);
  cbarg0.a = arg0;
  cbarg0.b = arg1;

  try {
    return addHeapObject(new Promise(cbarg0.bind(cbarg0)));
  } finally {
    cbarg0.a = cbarg0.b = 0;
  }
}

__exports.__wbg_new_ce158cf1048d4c17 = __wbg_new_ce158cf1048d4c17;

function __wbg_resolve_de6a9d3662905882(arg0) {
  return addHeapObject(Promise.resolve(getObject(arg0)));
}

__exports.__wbg_resolve_de6a9d3662905882 = __wbg_resolve_de6a9d3662905882;

function __wbg_then_3faaae6de0104bf6(arg0, arg1) {
  return addHeapObject(getObject(arg0).then(getObject(arg1)));
}

__exports.__wbg_then_3faaae6de0104bf6 = __wbg_then_3faaae6de0104bf6;

function __wbg_then_76e86e45033cabdf(arg0, arg1, arg2) {
  return addHeapObject(getObject(arg0).then(getObject(arg1), getObject(arg2)));
}

__exports.__wbg_then_76e86e45033cabdf = __wbg_then_76e86e45033cabdf;
var stack_pointer = 32;

function addBorrowedObject(obj) {
  if (stack_pointer == 1) throw new Error('out of js stack');
  heap[--stack_pointer] = obj;
  return stack_pointer;
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

function __wbindgen_string_new(p, l) {
  return addHeapObject(getStringFromWasm(p, l));
}

__exports.__wbindgen_string_new = __wbindgen_string_new;

function __wbindgen_string_get(i, len_ptr) {
  var obj = getObject(i);
  if (typeof obj !== 'string') return 0;
  var ptr = passStringToWasm(obj);
  getUint32Memory()[len_ptr / 4] = WASM_VECTOR_LEN;
  return ptr;
}

__exports.__wbindgen_string_get = __wbindgen_string_get;

function __wbindgen_debug_string(i, len_ptr) {
  var debug_str = function debug_str(val) {
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
      var _debug = '[';

      if (length > 0) {
        _debug += debug_str(val[0]);
      }

      for (var _i = 1; _i < length; _i++) {
        _debug += ', ' + debug_str(val[_i]);
      }

      _debug += ']';
      return _debug;
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
      return "".concat(val.name, ": ").concat(val.message, "\n        ").concat(val.stack);
    } // TODO we could test for more things here, like `Set`s and `Map`s.


    return className;
  };

  var toString = Object.prototype.toString;
  var val = getObject(i);
  var debug = debug_str(val);
  var ptr = passStringToWasm(debug);
  getUint32Memory()[len_ptr / 4] = WASM_VECTOR_LEN;
  return ptr;
}

__exports.__wbindgen_debug_string = __wbindgen_debug_string;

function __wbindgen_cb_drop(i) {
  var obj = takeObject(i).original;

  if (obj.cnt-- == 1) {
    obj.a = 0;
    return 1;
  }

  return 0;
}

__exports.__wbindgen_cb_drop = __wbindgen_cb_drop;

function __wbindgen_json_parse(ptr, len) {
  return addHeapObject(JSON.parse(getStringFromWasm(ptr, len)));
}

__exports.__wbindgen_json_parse = __wbindgen_json_parse;

function __wbindgen_json_serialize(idx, ptrptr) {
  var ptr = passStringToWasm(JSON.stringify(getObject(idx)));
  getUint32Memory()[ptrptr / 4] = ptr;
  return WASM_VECTOR_LEN;
}

__exports.__wbindgen_json_serialize = __wbindgen_json_serialize;

function __wbindgen_throw(ptr, len) {
  throw new Error(getStringFromWasm(ptr, len));
}

__exports.__wbindgen_throw = __wbindgen_throw;

function __wbindgen_closure_wrapper235(a, b, _ignored) {
  var f = wasm.__wbg_function_table.get(14);

  var d = wasm.__wbg_function_table.get(15);

  var cb = function cb(arg0) {
    this.cnt++;
    var a = this.a;
    this.a = 0;

    try {
      return f(a, b, addHeapObject(arg0));
    } finally {
      if (--this.cnt === 0) d(a, b);else this.a = a;
    }
  };

  cb.a = a;
  cb.cnt = 1;
  var real = cb.bind(cb);
  real.original = cb;
  return addHeapObject(real);
}

__exports.__wbindgen_closure_wrapper235 = __wbindgen_closure_wrapper235;

function freeContainerService(ptr) {
  wasm.__wbg_containerservice_free(ptr);
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
      freeContainerService(ptr);
    }
    /**
    * @param {any} emit
    * @returns {}
    */

  }]);

  function ContainerService(emit) {
    _classCallCheck(this, ContainerService);

    this.ptr = wasm.containerservice_new(addHeapObject(emit));
  }
  /**
  * @param {any} action_js
  * @returns {void}
  */


  _createClass(ContainerService, [{
    key: "dispatch",
    value: function dispatch(action_js) {
      try {
        return wasm.containerservice_dispatch(this.ptr, addBorrowedObject(action_js));
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
      return takeObject(wasm.containerservice_get_state(this.ptr));
    }
  }]);

  return ContainerService;
}();

exports.ContainerService = ContainerService;
__exports.ContainerService = ContainerService;

function __wbindgen_object_clone_ref(idx) {
  return addHeapObject(getObject(idx));
}

__exports.__wbindgen_object_clone_ref = __wbindgen_object_clone_ref;

function __wbindgen_object_drop_ref(i) {
  dropObject(i);
}

__exports.__wbindgen_object_drop_ref = __wbindgen_object_drop_ref;

function init(module) {
  var result;
  var imports = {
    './stremio_state_container_web': __exports
  };

  if (module instanceof URL || typeof module === 'string' || module instanceof Request) {
    var response = fetch(module);

    if (typeof WebAssembly.instantiateStreaming === 'function') {
      result = WebAssembly.instantiateStreaming(response, imports)["catch"](function (e) {
        console.warn("`WebAssembly.instantiateStreaming` failed. Assuming this is because your server does not serve wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);
        return response.then(function (r) {
          return r.arrayBuffer();
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
