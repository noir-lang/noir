/******/ (() => { // webpackBootstrap
/******/ 	var __webpack_modules__ = ({

/***/ "../source-resolver/lib-node/index.js":
/*!********************************************!*\
  !*** ../source-resolver/lib-node/index.js ***!
  \********************************************/
/***/ ((__unused_webpack_module, exports) => {

"use strict";


Object.defineProperty(exports, "__esModule", ({
  value: true
}));
exports.initializeResolver = exports.read_file = void 0;
var resolveFunction = null;
var read_file = function read_file(source_id) {
  if (resolveFunction) {
    var result = resolveFunction(source_id);
    if (typeof result === 'string') {
      return result;
    } else {
      throw new Error('Noir source resolver function MUST return String synchronously. Are you trying to return anything else, eg. `Promise`?');
    }
  } else {
    throw new Error('Not yet initialized. Use initializeResolver(() => string)');
  }
};
exports.read_file = read_file;
function initialize(noir_resolver) {
  if (typeof noir_resolver === 'function') {
    return noir_resolver;
  } else {
    throw new Error('Provided Noir Resolver is not a function, hint: use function(module_id) => NoirSource as second parameter');
  }
}
function initializeResolver(resolver) {
  resolveFunction = initialize(resolver);
}
exports.initializeResolver = initializeResolver;

/***/ }),

/***/ "../source-resolver/lib-node/index_node.js":
/*!*************************************************!*\
  !*** ../source-resolver/lib-node/index_node.js ***!
  \*************************************************/
/***/ ((__unused_webpack_module, exports, __webpack_require__) => {

"use strict";


/// <reference types="node" />
Object.defineProperty(exports, "__esModule", ({
  value: true
}));
exports.read_file = exports.initializeResolver = void 0;
var index_js_1 = __webpack_require__(/*! ./index.js */ "../source-resolver/lib-node/index.js");
Object.defineProperty(exports, "initializeResolver", ({
  enumerable: true,
  get: function get() {
    return index_js_1.initializeResolver;
  }
}));
Object.defineProperty(exports, "read_file", ({
  enumerable: true,
  get: function get() {
    return index_js_1.read_file;
  }
}));
(0, index_js_1.initializeResolver)(function (source_id) {
  var fileContent = '';
  try {
    // eslint-disable-next-line @typescript-eslint/no-var-requires
    var fs = __webpack_require__(/*! fs */ "fs");
    fileContent = fs.readFileSync(source_id, {
      encoding: 'utf8'
    });
  } catch (e) {
    console.log(e);
  }
  return fileContent;
});

/***/ }),

/***/ "./noir_wasm/cjs/index.js":
/*!********************************!*\
  !*** ./noir_wasm/cjs/index.js ***!
  \********************************/
/***/ ((module, __webpack_exports__, __webpack_require__) => {

"use strict";
__webpack_require__.a(module, async (__webpack_handle_async_dependencies__, __webpack_async_result__) => { try {
__webpack_require__.r(__webpack_exports__);
/* harmony export */ __webpack_require__.d(__webpack_exports__, {
/* harmony export */   __wbg_constructor_1292ee4141d8f79d: () => (/* reexport safe */ _index_bg_js__WEBPACK_IMPORTED_MODULE_0__.__wbg_constructor_1292ee4141d8f79d),
/* harmony export */   __wbg_constructor_2f52aa89ce97ba46: () => (/* reexport safe */ _index_bg_js__WEBPACK_IMPORTED_MODULE_0__.__wbg_constructor_2f52aa89ce97ba46),
/* harmony export */   __wbg_debug_efabe4eb183aa5d4: () => (/* reexport safe */ _index_bg_js__WEBPACK_IMPORTED_MODULE_0__.__wbg_debug_efabe4eb183aa5d4),
/* harmony export */   __wbg_error_50f42b952a595a23: () => (/* reexport safe */ _index_bg_js__WEBPACK_IMPORTED_MODULE_0__.__wbg_error_50f42b952a595a23),
/* harmony export */   __wbg_error_a7e23606158b68b9: () => (/* reexport safe */ _index_bg_js__WEBPACK_IMPORTED_MODULE_0__.__wbg_error_a7e23606158b68b9),
/* harmony export */   __wbg_error_f851667af71bcfc6: () => (/* reexport safe */ _index_bg_js__WEBPACK_IMPORTED_MODULE_0__.__wbg_error_f851667af71bcfc6),
/* harmony export */   __wbg_info_24d8f53d98f12b95: () => (/* reexport safe */ _index_bg_js__WEBPACK_IMPORTED_MODULE_0__.__wbg_info_24d8f53d98f12b95),
/* harmony export */   __wbg_log_9b164efbe6db702f: () => (/* reexport safe */ _index_bg_js__WEBPACK_IMPORTED_MODULE_0__.__wbg_log_9b164efbe6db702f),
/* harmony export */   __wbg_new_abda76e883ba8a5f: () => (/* reexport safe */ _index_bg_js__WEBPACK_IMPORTED_MODULE_0__.__wbg_new_abda76e883ba8a5f),
/* harmony export */   __wbg_parse_76a8a18ca3f8730b: () => (/* reexport safe */ _index_bg_js__WEBPACK_IMPORTED_MODULE_0__.__wbg_parse_76a8a18ca3f8730b),
/* harmony export */   __wbg_readfile_698fcfe83a414130: () => (/* reexport safe */ _index_bg_js__WEBPACK_IMPORTED_MODULE_0__.__wbg_readfile_698fcfe83a414130),
/* harmony export */   __wbg_set_07da13cc24b69217: () => (/* reexport safe */ _index_bg_js__WEBPACK_IMPORTED_MODULE_0__.__wbg_set_07da13cc24b69217),
/* harmony export */   __wbg_set_wasm: () => (/* reexport safe */ _index_bg_js__WEBPACK_IMPORTED_MODULE_0__.__wbg_set_wasm),
/* harmony export */   __wbg_stack_658279fe44541cf6: () => (/* reexport safe */ _index_bg_js__WEBPACK_IMPORTED_MODULE_0__.__wbg_stack_658279fe44541cf6),
/* harmony export */   __wbg_stringify_d06ad2addc54d51e: () => (/* reexport safe */ _index_bg_js__WEBPACK_IMPORTED_MODULE_0__.__wbg_stringify_d06ad2addc54d51e),
/* harmony export */   __wbg_warn_8342bfbc6028193a: () => (/* reexport safe */ _index_bg_js__WEBPACK_IMPORTED_MODULE_0__.__wbg_warn_8342bfbc6028193a),
/* harmony export */   __wbindgen_debug_string: () => (/* reexport safe */ _index_bg_js__WEBPACK_IMPORTED_MODULE_0__.__wbindgen_debug_string),
/* harmony export */   __wbindgen_is_undefined: () => (/* reexport safe */ _index_bg_js__WEBPACK_IMPORTED_MODULE_0__.__wbindgen_is_undefined),
/* harmony export */   __wbindgen_object_drop_ref: () => (/* reexport safe */ _index_bg_js__WEBPACK_IMPORTED_MODULE_0__.__wbindgen_object_drop_ref),
/* harmony export */   __wbindgen_string_get: () => (/* reexport safe */ _index_bg_js__WEBPACK_IMPORTED_MODULE_0__.__wbindgen_string_get),
/* harmony export */   __wbindgen_string_new: () => (/* reexport safe */ _index_bg_js__WEBPACK_IMPORTED_MODULE_0__.__wbindgen_string_new),
/* harmony export */   __wbindgen_throw: () => (/* reexport safe */ _index_bg_js__WEBPACK_IMPORTED_MODULE_0__.__wbindgen_throw),
/* harmony export */   acir_read_bytes: () => (/* reexport safe */ _index_bg_js__WEBPACK_IMPORTED_MODULE_0__.acir_read_bytes),
/* harmony export */   acir_write_bytes: () => (/* reexport safe */ _index_bg_js__WEBPACK_IMPORTED_MODULE_0__.acir_write_bytes),
/* harmony export */   build_info: () => (/* reexport safe */ _index_bg_js__WEBPACK_IMPORTED_MODULE_0__.build_info),
/* harmony export */   compile: () => (/* reexport safe */ _index_bg_js__WEBPACK_IMPORTED_MODULE_0__.compile),
/* harmony export */   init_log_level: () => (/* reexport safe */ _index_bg_js__WEBPACK_IMPORTED_MODULE_0__.init_log_level)
/* harmony export */ });
/* harmony import */ var _index_bg_wasm__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! ./index_bg.wasm */ "./noir_wasm/cjs/index_bg.wasm");
/* harmony import */ var _index_bg_js__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! ./index_bg.js */ "./noir_wasm/cjs/index_bg.js");
var __webpack_async_dependencies__ = __webpack_handle_async_dependencies__([_index_bg_wasm__WEBPACK_IMPORTED_MODULE_1__]);
_index_bg_wasm__WEBPACK_IMPORTED_MODULE_1__ = (__webpack_async_dependencies__.then ? (await __webpack_async_dependencies__)() : __webpack_async_dependencies__)[0];


(0,_index_bg_js__WEBPACK_IMPORTED_MODULE_0__.__wbg_set_wasm)(_index_bg_wasm__WEBPACK_IMPORTED_MODULE_1__);

__webpack_async_result__();
} catch(e) { __webpack_async_result__(e); } });

/***/ }),

/***/ "./noir_wasm/cjs/index_bg.js":
/*!***********************************!*\
  !*** ./noir_wasm/cjs/index_bg.js ***!
  \***********************************/
/***/ ((module, __webpack_exports__, __webpack_require__) => {

"use strict";
__webpack_require__.r(__webpack_exports__);
/* harmony export */ __webpack_require__.d(__webpack_exports__, {
/* harmony export */   __wbg_constructor_1292ee4141d8f79d: () => (/* binding */ __wbg_constructor_1292ee4141d8f79d),
/* harmony export */   __wbg_constructor_2f52aa89ce97ba46: () => (/* binding */ __wbg_constructor_2f52aa89ce97ba46),
/* harmony export */   __wbg_debug_efabe4eb183aa5d4: () => (/* binding */ __wbg_debug_efabe4eb183aa5d4),
/* harmony export */   __wbg_error_50f42b952a595a23: () => (/* binding */ __wbg_error_50f42b952a595a23),
/* harmony export */   __wbg_error_a7e23606158b68b9: () => (/* binding */ __wbg_error_a7e23606158b68b9),
/* harmony export */   __wbg_error_f851667af71bcfc6: () => (/* binding */ __wbg_error_f851667af71bcfc6),
/* harmony export */   __wbg_info_24d8f53d98f12b95: () => (/* binding */ __wbg_info_24d8f53d98f12b95),
/* harmony export */   __wbg_log_9b164efbe6db702f: () => (/* binding */ __wbg_log_9b164efbe6db702f),
/* harmony export */   __wbg_new_abda76e883ba8a5f: () => (/* binding */ __wbg_new_abda76e883ba8a5f),
/* harmony export */   __wbg_parse_76a8a18ca3f8730b: () => (/* binding */ __wbg_parse_76a8a18ca3f8730b),
/* harmony export */   __wbg_readfile_698fcfe83a414130: () => (/* binding */ __wbg_readfile_698fcfe83a414130),
/* harmony export */   __wbg_set_07da13cc24b69217: () => (/* binding */ __wbg_set_07da13cc24b69217),
/* harmony export */   __wbg_set_wasm: () => (/* binding */ __wbg_set_wasm),
/* harmony export */   __wbg_stack_658279fe44541cf6: () => (/* binding */ __wbg_stack_658279fe44541cf6),
/* harmony export */   __wbg_stringify_d06ad2addc54d51e: () => (/* binding */ __wbg_stringify_d06ad2addc54d51e),
/* harmony export */   __wbg_warn_8342bfbc6028193a: () => (/* binding */ __wbg_warn_8342bfbc6028193a),
/* harmony export */   __wbindgen_debug_string: () => (/* binding */ __wbindgen_debug_string),
/* harmony export */   __wbindgen_is_undefined: () => (/* binding */ __wbindgen_is_undefined),
/* harmony export */   __wbindgen_object_drop_ref: () => (/* binding */ __wbindgen_object_drop_ref),
/* harmony export */   __wbindgen_string_get: () => (/* binding */ __wbindgen_string_get),
/* harmony export */   __wbindgen_string_new: () => (/* binding */ __wbindgen_string_new),
/* harmony export */   __wbindgen_throw: () => (/* binding */ __wbindgen_throw),
/* harmony export */   acir_read_bytes: () => (/* binding */ acir_read_bytes),
/* harmony export */   acir_write_bytes: () => (/* binding */ acir_write_bytes),
/* harmony export */   build_info: () => (/* binding */ build_info),
/* harmony export */   compile: () => (/* binding */ compile),
/* harmony export */   init_log_level: () => (/* binding */ init_log_level)
/* harmony export */ });
Object(function webpackMissingModule() { var e = new Error("Cannot find module '@noir-lang/source-resolver'"); e.code = 'MODULE_NOT_FOUND'; throw e; }());
/* module decorator */ module = __webpack_require__.hmd(module);
function _typeof(o) { "@babel/helpers - typeof"; return _typeof = "function" == typeof Symbol && "symbol" == typeof Symbol.iterator ? function (o) { return typeof o; } : function (o) { return o && "function" == typeof Symbol && o.constructor === Symbol && o !== Symbol.prototype ? "symbol" : typeof o; }, _typeof(o); }

var wasm;
function __wbg_set_wasm(val) {
  wasm = val;
}
var heap = new Array(128).fill(undefined);
heap.push(undefined, null, true, false);
function getObject(idx) {
  return heap[idx];
}
var heap_next = heap.length;
function dropObject(idx) {
  if (idx < 132) return;
  heap[idx] = heap_next;
  heap_next = idx;
}
function takeObject(idx) {
  var ret = getObject(idx);
  dropObject(idx);
  return ret;
}
function _assertBoolean(n) {
  if (typeof n !== 'boolean') {
    throw new Error('expected a boolean argument');
  }
}
var lTextDecoder = typeof TextDecoder === 'undefined' ? (0, module.require)('util').TextDecoder : TextDecoder;
var cachedTextDecoder = new lTextDecoder('utf-8', {
  ignoreBOM: true,
  fatal: true
});
cachedTextDecoder.decode();
var cachedUint8Memory0 = null;
function getUint8Memory0() {
  if (cachedUint8Memory0 === null || cachedUint8Memory0.byteLength === 0) {
    cachedUint8Memory0 = new Uint8Array(wasm.memory.buffer);
  }
  return cachedUint8Memory0;
}
function getStringFromWasm0(ptr, len) {
  ptr = ptr >>> 0;
  return cachedTextDecoder.decode(getUint8Memory0().subarray(ptr, ptr + len));
}
function addHeapObject(obj) {
  if (heap_next === heap.length) heap.push(heap.length + 1);
  var idx = heap_next;
  heap_next = heap[idx];
  if (typeof heap_next !== 'number') throw new Error('corrupt heap');
  heap[idx] = obj;
  return idx;
}
var WASM_VECTOR_LEN = 0;
var lTextEncoder = typeof TextEncoder === 'undefined' ? (0, module.require)('util').TextEncoder : TextEncoder;
var cachedTextEncoder = new lTextEncoder('utf-8');
var encodeString = typeof cachedTextEncoder.encodeInto === 'function' ? function (arg, view) {
  return cachedTextEncoder.encodeInto(arg, view);
} : function (arg, view) {
  var buf = cachedTextEncoder.encode(arg);
  view.set(buf);
  return {
    read: arg.length,
    written: buf.length
  };
};
function passStringToWasm0(arg, malloc, realloc) {
  if (typeof arg !== 'string') throw new Error('expected a string argument');
  if (realloc === undefined) {
    var buf = cachedTextEncoder.encode(arg);
    var _ptr = malloc(buf.length) >>> 0;
    getUint8Memory0().subarray(_ptr, _ptr + buf.length).set(buf);
    WASM_VECTOR_LEN = buf.length;
    return _ptr;
  }
  var len = arg.length;
  var ptr = malloc(len) >>> 0;
  var mem = getUint8Memory0();
  var offset = 0;
  for (; offset < len; offset++) {
    var code = arg.charCodeAt(offset);
    if (code > 0x7F) break;
    mem[ptr + offset] = code;
  }
  if (offset !== len) {
    if (offset !== 0) {
      arg = arg.slice(offset);
    }
    ptr = realloc(ptr, len, len = offset + arg.length * 3) >>> 0;
    var view = getUint8Memory0().subarray(ptr + offset, ptr + len);
    var ret = encodeString(arg, view);
    if (ret.read !== arg.length) throw new Error('failed to pass whole string');
    offset += ret.written;
  }
  WASM_VECTOR_LEN = offset;
  return ptr;
}
function isLikeNone(x) {
  return x === undefined || x === null;
}
var cachedInt32Memory0 = null;
function getInt32Memory0() {
  if (cachedInt32Memory0 === null || cachedInt32Memory0.byteLength === 0) {
    cachedInt32Memory0 = new Int32Array(wasm.memory.buffer);
  }
  return cachedInt32Memory0;
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
  }
  // objects
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
  }
  // Test for built-in
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
  }
  // errors
  if (val instanceof Error) {
    return "".concat(val.name, ": ").concat(val.message, "\n").concat(val.stack);
  }
  // TODO we could test for more things here, like `Set`s and `Map`s.
  return className;
}
/**
* @param {string} level
*/
function init_log_level(level) {
  var ptr0 = passStringToWasm0(level, wasm.__wbindgen_export_0, wasm.__wbindgen_export_1);
  var len0 = WASM_VECTOR_LEN;
  wasm.init_log_level(ptr0, len0);
}

/**
* @returns {any}
*/
function build_info() {
  var ret = wasm.build_info();
  return takeObject(ret);
}
function logError(f, args) {
  try {
    return f.apply(this, args);
  } catch (e) {
    var error = function () {
      try {
        return e instanceof Error ? "".concat(e.message, "\n\nStack:\n").concat(e.stack) : e.toString();
      } catch (_) {
        return "<failed to stringify thrown value>";
      }
    }();
    console.error("wasm-bindgen: imported JS function that was not marked as `catch` threw an error:", error);
    throw e;
  }
}
/**
* @param {string} entry_point
* @param {boolean | undefined} contracts
* @param {DependencyGraph | undefined} dependency_graph
* @returns {CompileResult}
*/
function compile(entry_point, contracts, dependency_graph) {
  try {
    var retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
    var ptr0 = passStringToWasm0(entry_point, wasm.__wbindgen_export_0, wasm.__wbindgen_export_1);
    var len0 = WASM_VECTOR_LEN;
    if (!isLikeNone(contracts)) {
      _assertBoolean(contracts);
    }
    wasm.compile(retptr, ptr0, len0, isLikeNone(contracts) ? 0xFFFFFF : contracts ? 1 : 0, isLikeNone(dependency_graph) ? 0 : addHeapObject(dependency_graph));
    var r0 = getInt32Memory0()[retptr / 4 + 0];
    var r1 = getInt32Memory0()[retptr / 4 + 1];
    var r2 = getInt32Memory0()[retptr / 4 + 2];
    if (r2) {
      throw takeObject(r1);
    }
    return takeObject(r0);
  } finally {
    wasm.__wbindgen_add_to_stack_pointer(16);
  }
}
function handleError(f, args) {
  try {
    return f.apply(this, args);
  } catch (e) {
    wasm.__wbindgen_export_2(addHeapObject(e));
  }
}
function passArray8ToWasm0(arg, malloc) {
  var ptr = malloc(arg.length * 1) >>> 0;
  getUint8Memory0().set(arg, ptr / 1);
  WASM_VECTOR_LEN = arg.length;
  return ptr;
}
/**
* @param {Uint8Array} bytes
* @returns {any}
*/
function acir_read_bytes(bytes) {
  var ptr0 = passArray8ToWasm0(bytes, wasm.__wbindgen_export_0);
  var len0 = WASM_VECTOR_LEN;
  var ret = wasm.acir_read_bytes(ptr0, len0);
  return takeObject(ret);
}
function getArrayU8FromWasm0(ptr, len) {
  ptr = ptr >>> 0;
  return getUint8Memory0().subarray(ptr / 1, ptr / 1 + len);
}
/**
* @param {any} acir
* @returns {Uint8Array}
*/
function acir_write_bytes(acir) {
  try {
    var retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
    wasm.acir_write_bytes(retptr, addHeapObject(acir));
    var r0 = getInt32Memory0()[retptr / 4 + 0];
    var r1 = getInt32Memory0()[retptr / 4 + 1];
    var v1 = getArrayU8FromWasm0(r0, r1).slice();
    wasm.__wbindgen_export_3(r0, r1 * 1);
    return v1;
  } finally {
    wasm.__wbindgen_add_to_stack_pointer(16);
  }
}
function __wbindgen_object_drop_ref(arg0) {
  takeObject(arg0);
}
;
function __wbg_constructor_2f52aa89ce97ba46() {
  return logError(function (arg0) {
    var ret = new Error(takeObject(arg0));
    return addHeapObject(ret);
  }, arguments);
}
;
function __wbg_constructor_1292ee4141d8f79d() {
  return logError(function () {
    var ret = new Object();
    return addHeapObject(ret);
  }, arguments);
}
;
function __wbg_readfile_698fcfe83a414130() {
  return handleError(function (arg0, arg1, arg2) {
    var ret = Object(function webpackMissingModule() { var e = new Error("Cannot find module '@noir-lang/source-resolver'"); e.code = 'MODULE_NOT_FOUND'; throw e; }())(getStringFromWasm0(arg1, arg2));
    var ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_0, wasm.__wbindgen_export_1);
    var len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
  }, arguments);
}
;
function __wbindgen_is_undefined(arg0) {
  var ret = getObject(arg0) === undefined;
  _assertBoolean(ret);
  return ret;
}
;
function __wbindgen_string_new(arg0, arg1) {
  var ret = getStringFromWasm0(arg0, arg1);
  return addHeapObject(ret);
}
;
function __wbg_new_abda76e883ba8a5f() {
  return logError(function () {
    var ret = new Error();
    return addHeapObject(ret);
  }, arguments);
}
;
function __wbg_stack_658279fe44541cf6() {
  return logError(function (arg0, arg1) {
    var ret = getObject(arg1).stack;
    var ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_0, wasm.__wbindgen_export_1);
    var len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
  }, arguments);
}
;
function __wbg_error_f851667af71bcfc6() {
  return logError(function (arg0, arg1) {
    var deferred0_0;
    var deferred0_1;
    try {
      deferred0_0 = arg0;
      deferred0_1 = arg1;
      console.error(getStringFromWasm0(arg0, arg1));
    } finally {
      wasm.__wbindgen_export_3(deferred0_0, deferred0_1);
    }
  }, arguments);
}
;
function __wbg_debug_efabe4eb183aa5d4() {
  return logError(function (arg0, arg1, arg2, arg3) {
    console.debug(getObject(arg0), getObject(arg1), getObject(arg2), getObject(arg3));
  }, arguments);
}
;
function __wbg_error_a7e23606158b68b9() {
  return logError(function (arg0) {
    console.error(getObject(arg0));
  }, arguments);
}
;
function __wbg_error_50f42b952a595a23() {
  return logError(function (arg0, arg1, arg2, arg3) {
    console.error(getObject(arg0), getObject(arg1), getObject(arg2), getObject(arg3));
  }, arguments);
}
;
function __wbg_info_24d8f53d98f12b95() {
  return logError(function (arg0, arg1, arg2, arg3) {
    console.info(getObject(arg0), getObject(arg1), getObject(arg2), getObject(arg3));
  }, arguments);
}
;
function __wbg_log_9b164efbe6db702f() {
  return logError(function (arg0, arg1, arg2, arg3) {
    console.log(getObject(arg0), getObject(arg1), getObject(arg2), getObject(arg3));
  }, arguments);
}
;
function __wbg_warn_8342bfbc6028193a() {
  return logError(function (arg0, arg1, arg2, arg3) {
    console.warn(getObject(arg0), getObject(arg1), getObject(arg2), getObject(arg3));
  }, arguments);
}
;
function __wbindgen_string_get(arg0, arg1) {
  var obj = getObject(arg1);
  var ret = typeof obj === 'string' ? obj : undefined;
  var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_export_0, wasm.__wbindgen_export_1);
  var len1 = WASM_VECTOR_LEN;
  getInt32Memory0()[arg0 / 4 + 1] = len1;
  getInt32Memory0()[arg0 / 4 + 0] = ptr1;
}
;
function __wbg_parse_76a8a18ca3f8730b() {
  return handleError(function (arg0, arg1) {
    var ret = JSON.parse(getStringFromWasm0(arg0, arg1));
    return addHeapObject(ret);
  }, arguments);
}
;
function __wbg_stringify_d06ad2addc54d51e() {
  return handleError(function (arg0) {
    var ret = JSON.stringify(getObject(arg0));
    return addHeapObject(ret);
  }, arguments);
}
;
function __wbg_set_07da13cc24b69217() {
  return handleError(function (arg0, arg1, arg2) {
    var ret = Reflect.set(getObject(arg0), getObject(arg1), getObject(arg2));
    _assertBoolean(ret);
    return ret;
  }, arguments);
}
;
function __wbindgen_debug_string(arg0, arg1) {
  var ret = debugString(getObject(arg1));
  var ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export_0, wasm.__wbindgen_export_1);
  var len1 = WASM_VECTOR_LEN;
  getInt32Memory0()[arg0 / 4 + 1] = len1;
  getInt32Memory0()[arg0 / 4 + 0] = ptr1;
}
;
function __wbindgen_throw(arg0, arg1) {
  throw new Error(getStringFromWasm0(arg0, arg1));
}
;

/***/ }),

/***/ "./noir_wasm/src/noir/dependencies/dependency-manager.ts":
/*!***************************************************************!*\
  !*** ./noir_wasm/src/noir/dependencies/dependency-manager.ts ***!
  \***************************************************************/
/***/ ((__unused_webpack_module, __webpack_exports__, __webpack_require__) => {

"use strict";
__webpack_require__.r(__webpack_exports__);
/* harmony export */ __webpack_require__.d(__webpack_exports__, {
/* harmony export */   NoirDependencyManager: () => (/* binding */ NoirDependencyManager)
/* harmony export */ });
/* harmony import */ var path__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! path */ "path");
/* harmony import */ var path__WEBPACK_IMPORTED_MODULE_0___default = /*#__PURE__*/__webpack_require__.n(path__WEBPACK_IMPORTED_MODULE_0__);
function _typeof(o) { "@babel/helpers - typeof"; return _typeof = "function" == typeof Symbol && "symbol" == typeof Symbol.iterator ? function (o) { return typeof o; } : function (o) { return o && "function" == typeof Symbol && o.constructor === Symbol && o !== Symbol.prototype ? "symbol" : typeof o; }, _typeof(o); }
function _createForOfIteratorHelper(o, allowArrayLike) { var it = typeof Symbol !== "undefined" && o[Symbol.iterator] || o["@@iterator"]; if (!it) { if (Array.isArray(o) || (it = _unsupportedIterableToArray(o)) || allowArrayLike && o && typeof o.length === "number") { if (it) o = it; var i = 0; var F = function F() {}; return { s: F, n: function n() { if (i >= o.length) return { done: true }; return { done: false, value: o[i++] }; }, e: function e(_e) { throw _e; }, f: F }; } throw new TypeError("Invalid attempt to iterate non-iterable instance.\nIn order to be iterable, non-array objects must have a [Symbol.iterator]() method."); } var normalCompletion = true, didErr = false, err; return { s: function s() { it = it.call(o); }, n: function n() { var step = it.next(); normalCompletion = step.done; return step; }, e: function e(_e2) { didErr = true; err = _e2; }, f: function f() { try { if (!normalCompletion && it["return"] != null) it["return"](); } finally { if (didErr) throw err; } } }; }
function _toArray(arr) { return _arrayWithHoles(arr) || _iterableToArray(arr) || _unsupportedIterableToArray(arr) || _nonIterableRest(); }
function _toConsumableArray(arr) { return _arrayWithoutHoles(arr) || _iterableToArray(arr) || _unsupportedIterableToArray(arr) || _nonIterableSpread(); }
function _nonIterableSpread() { throw new TypeError("Invalid attempt to spread non-iterable instance.\nIn order to be iterable, non-array objects must have a [Symbol.iterator]() method."); }
function _iterableToArray(iter) { if (typeof Symbol !== "undefined" && iter[Symbol.iterator] != null || iter["@@iterator"] != null) return Array.from(iter); }
function _arrayWithoutHoles(arr) { if (Array.isArray(arr)) return _arrayLikeToArray(arr); }
function _regeneratorRuntime() { "use strict"; /*! regenerator-runtime -- Copyright (c) 2014-present, Facebook, Inc. -- license (MIT): https://github.com/facebook/regenerator/blob/main/LICENSE */ _regeneratorRuntime = function _regeneratorRuntime() { return e; }; var t, e = {}, r = Object.prototype, n = r.hasOwnProperty, o = Object.defineProperty || function (t, e, r) { t[e] = r.value; }, i = "function" == typeof Symbol ? Symbol : {}, a = i.iterator || "@@iterator", c = i.asyncIterator || "@@asyncIterator", u = i.toStringTag || "@@toStringTag"; function define(t, e, r) { return Object.defineProperty(t, e, { value: r, enumerable: !0, configurable: !0, writable: !0 }), t[e]; } try { define({}, ""); } catch (t) { define = function define(t, e, r) { return t[e] = r; }; } function wrap(t, e, r, n) { var i = e && e.prototype instanceof Generator ? e : Generator, a = Object.create(i.prototype), c = new Context(n || []); return o(a, "_invoke", { value: makeInvokeMethod(t, r, c) }), a; } function tryCatch(t, e, r) { try { return { type: "normal", arg: t.call(e, r) }; } catch (t) { return { type: "throw", arg: t }; } } e.wrap = wrap; var h = "suspendedStart", l = "suspendedYield", f = "executing", s = "completed", y = {}; function Generator() {} function GeneratorFunction() {} function GeneratorFunctionPrototype() {} var p = {}; define(p, a, function () { return this; }); var d = Object.getPrototypeOf, v = d && d(d(values([]))); v && v !== r && n.call(v, a) && (p = v); var g = GeneratorFunctionPrototype.prototype = Generator.prototype = Object.create(p); function defineIteratorMethods(t) { ["next", "throw", "return"].forEach(function (e) { define(t, e, function (t) { return this._invoke(e, t); }); }); } function AsyncIterator(t, e) { function invoke(r, o, i, a) { var c = tryCatch(t[r], t, o); if ("throw" !== c.type) { var u = c.arg, h = u.value; return h && "object" == _typeof(h) && n.call(h, "__await") ? e.resolve(h.__await).then(function (t) { invoke("next", t, i, a); }, function (t) { invoke("throw", t, i, a); }) : e.resolve(h).then(function (t) { u.value = t, i(u); }, function (t) { return invoke("throw", t, i, a); }); } a(c.arg); } var r; o(this, "_invoke", { value: function value(t, n) { function callInvokeWithMethodAndArg() { return new e(function (e, r) { invoke(t, n, e, r); }); } return r = r ? r.then(callInvokeWithMethodAndArg, callInvokeWithMethodAndArg) : callInvokeWithMethodAndArg(); } }); } function makeInvokeMethod(e, r, n) { var o = h; return function (i, a) { if (o === f) throw new Error("Generator is already running"); if (o === s) { if ("throw" === i) throw a; return { value: t, done: !0 }; } for (n.method = i, n.arg = a;;) { var c = n.delegate; if (c) { var u = maybeInvokeDelegate(c, n); if (u) { if (u === y) continue; return u; } } if ("next" === n.method) n.sent = n._sent = n.arg;else if ("throw" === n.method) { if (o === h) throw o = s, n.arg; n.dispatchException(n.arg); } else "return" === n.method && n.abrupt("return", n.arg); o = f; var p = tryCatch(e, r, n); if ("normal" === p.type) { if (o = n.done ? s : l, p.arg === y) continue; return { value: p.arg, done: n.done }; } "throw" === p.type && (o = s, n.method = "throw", n.arg = p.arg); } }; } function maybeInvokeDelegate(e, r) { var n = r.method, o = e.iterator[n]; if (o === t) return r.delegate = null, "throw" === n && e.iterator["return"] && (r.method = "return", r.arg = t, maybeInvokeDelegate(e, r), "throw" === r.method) || "return" !== n && (r.method = "throw", r.arg = new TypeError("The iterator does not provide a '" + n + "' method")), y; var i = tryCatch(o, e.iterator, r.arg); if ("throw" === i.type) return r.method = "throw", r.arg = i.arg, r.delegate = null, y; var a = i.arg; return a ? a.done ? (r[e.resultName] = a.value, r.next = e.nextLoc, "return" !== r.method && (r.method = "next", r.arg = t), r.delegate = null, y) : a : (r.method = "throw", r.arg = new TypeError("iterator result is not an object"), r.delegate = null, y); } function pushTryEntry(t) { var e = { tryLoc: t[0] }; 1 in t && (e.catchLoc = t[1]), 2 in t && (e.finallyLoc = t[2], e.afterLoc = t[3]), this.tryEntries.push(e); } function resetTryEntry(t) { var e = t.completion || {}; e.type = "normal", delete e.arg, t.completion = e; } function Context(t) { this.tryEntries = [{ tryLoc: "root" }], t.forEach(pushTryEntry, this), this.reset(!0); } function values(e) { if (e || "" === e) { var r = e[a]; if (r) return r.call(e); if ("function" == typeof e.next) return e; if (!isNaN(e.length)) { var o = -1, i = function next() { for (; ++o < e.length;) if (n.call(e, o)) return next.value = e[o], next.done = !1, next; return next.value = t, next.done = !0, next; }; return i.next = i; } } throw new TypeError(_typeof(e) + " is not iterable"); } return GeneratorFunction.prototype = GeneratorFunctionPrototype, o(g, "constructor", { value: GeneratorFunctionPrototype, configurable: !0 }), o(GeneratorFunctionPrototype, "constructor", { value: GeneratorFunction, configurable: !0 }), GeneratorFunction.displayName = define(GeneratorFunctionPrototype, u, "GeneratorFunction"), e.isGeneratorFunction = function (t) { var e = "function" == typeof t && t.constructor; return !!e && (e === GeneratorFunction || "GeneratorFunction" === (e.displayName || e.name)); }, e.mark = function (t) { return Object.setPrototypeOf ? Object.setPrototypeOf(t, GeneratorFunctionPrototype) : (t.__proto__ = GeneratorFunctionPrototype, define(t, u, "GeneratorFunction")), t.prototype = Object.create(g), t; }, e.awrap = function (t) { return { __await: t }; }, defineIteratorMethods(AsyncIterator.prototype), define(AsyncIterator.prototype, c, function () { return this; }), e.AsyncIterator = AsyncIterator, e.async = function (t, r, n, o, i) { void 0 === i && (i = Promise); var a = new AsyncIterator(wrap(t, r, n, o), i); return e.isGeneratorFunction(r) ? a : a.next().then(function (t) { return t.done ? t.value : a.next(); }); }, defineIteratorMethods(g), define(g, u, "Generator"), define(g, a, function () { return this; }), define(g, "toString", function () { return "[object Generator]"; }), e.keys = function (t) { var e = Object(t), r = []; for (var n in e) r.push(n); return r.reverse(), function next() { for (; r.length;) { var t = r.pop(); if (t in e) return next.value = t, next.done = !1, next; } return next.done = !0, next; }; }, e.values = values, Context.prototype = { constructor: Context, reset: function reset(e) { if (this.prev = 0, this.next = 0, this.sent = this._sent = t, this.done = !1, this.delegate = null, this.method = "next", this.arg = t, this.tryEntries.forEach(resetTryEntry), !e) for (var r in this) "t" === r.charAt(0) && n.call(this, r) && !isNaN(+r.slice(1)) && (this[r] = t); }, stop: function stop() { this.done = !0; var t = this.tryEntries[0].completion; if ("throw" === t.type) throw t.arg; return this.rval; }, dispatchException: function dispatchException(e) { if (this.done) throw e; var r = this; function handle(n, o) { return a.type = "throw", a.arg = e, r.next = n, o && (r.method = "next", r.arg = t), !!o; } for (var o = this.tryEntries.length - 1; o >= 0; --o) { var i = this.tryEntries[o], a = i.completion; if ("root" === i.tryLoc) return handle("end"); if (i.tryLoc <= this.prev) { var c = n.call(i, "catchLoc"), u = n.call(i, "finallyLoc"); if (c && u) { if (this.prev < i.catchLoc) return handle(i.catchLoc, !0); if (this.prev < i.finallyLoc) return handle(i.finallyLoc); } else if (c) { if (this.prev < i.catchLoc) return handle(i.catchLoc, !0); } else { if (!u) throw new Error("try statement without catch or finally"); if (this.prev < i.finallyLoc) return handle(i.finallyLoc); } } } }, abrupt: function abrupt(t, e) { for (var r = this.tryEntries.length - 1; r >= 0; --r) { var o = this.tryEntries[r]; if (o.tryLoc <= this.prev && n.call(o, "finallyLoc") && this.prev < o.finallyLoc) { var i = o; break; } } i && ("break" === t || "continue" === t) && i.tryLoc <= e && e <= i.finallyLoc && (i = null); var a = i ? i.completion : {}; return a.type = t, a.arg = e, i ? (this.method = "next", this.next = i.finallyLoc, y) : this.complete(a); }, complete: function complete(t, e) { if ("throw" === t.type) throw t.arg; return "break" === t.type || "continue" === t.type ? this.next = t.arg : "return" === t.type ? (this.rval = this.arg = t.arg, this.method = "return", this.next = "end") : "normal" === t.type && e && (this.next = e), y; }, finish: function finish(t) { for (var e = this.tryEntries.length - 1; e >= 0; --e) { var r = this.tryEntries[e]; if (r.finallyLoc === t) return this.complete(r.completion, r.afterLoc), resetTryEntry(r), y; } }, "catch": function _catch(t) { for (var e = this.tryEntries.length - 1; e >= 0; --e) { var r = this.tryEntries[e]; if (r.tryLoc === t) { var n = r.completion; if ("throw" === n.type) { var o = n.arg; resetTryEntry(r); } return o; } } throw new Error("illegal catch attempt"); }, delegateYield: function delegateYield(e, r, n) { return this.delegate = { iterator: values(e), resultName: r, nextLoc: n }, "next" === this.method && (this.arg = t), y; } }, e; }
function asyncGeneratorStep(gen, resolve, reject, _next, _throw, key, arg) { try { var info = gen[key](arg); var value = info.value; } catch (error) { reject(error); return; } if (info.done) { resolve(value); } else { Promise.resolve(value).then(_next, _throw); } }
function _asyncToGenerator(fn) { return function () { var self = this, args = arguments; return new Promise(function (resolve, reject) { var gen = fn.apply(self, args); function _next(value) { asyncGeneratorStep(gen, resolve, reject, _next, _throw, "next", value); } function _throw(err) { asyncGeneratorStep(gen, resolve, reject, _next, _throw, "throw", err); } _next(undefined); }); }; }
function _slicedToArray(arr, i) { return _arrayWithHoles(arr) || _iterableToArrayLimit(arr, i) || _unsupportedIterableToArray(arr, i) || _nonIterableRest(); }
function _nonIterableRest() { throw new TypeError("Invalid attempt to destructure non-iterable instance.\nIn order to be iterable, non-array objects must have a [Symbol.iterator]() method."); }
function _unsupportedIterableToArray(o, minLen) { if (!o) return; if (typeof o === "string") return _arrayLikeToArray(o, minLen); var n = Object.prototype.toString.call(o).slice(8, -1); if (n === "Object" && o.constructor) n = o.constructor.name; if (n === "Map" || n === "Set") return Array.from(o); if (n === "Arguments" || /^(?:Ui|I)nt(?:8|16|32)(?:Clamped)?Array$/.test(n)) return _arrayLikeToArray(o, minLen); }
function _arrayLikeToArray(arr, len) { if (len == null || len > arr.length) len = arr.length; for (var i = 0, arr2 = new Array(len); i < len; i++) arr2[i] = arr[i]; return arr2; }
function _iterableToArrayLimit(r, l) { var t = null == r ? null : "undefined" != typeof Symbol && r[Symbol.iterator] || r["@@iterator"]; if (null != t) { var e, n, i, u, a = [], f = !0, o = !1; try { if (i = (t = t.call(r)).next, 0 === l) { if (Object(t) !== t) return; f = !1; } else for (; !(f = (e = i.call(t)).done) && (a.push(e.value), a.length !== l); f = !0); } catch (r) { o = !0, n = r; } finally { try { if (!f && null != t["return"] && (u = t["return"](), Object(u) !== u)) return; } finally { if (o) throw n; } } return a; } }
function _arrayWithHoles(arr) { if (Array.isArray(arr)) return arr; }
function _classCallCheck(instance, Constructor) { if (!(instance instanceof Constructor)) { throw new TypeError("Cannot call a class as a function"); } }
function _defineProperties(target, props) { for (var i = 0; i < props.length; i++) { var descriptor = props[i]; descriptor.enumerable = descriptor.enumerable || false; descriptor.configurable = true; if ("value" in descriptor) descriptor.writable = true; Object.defineProperty(target, _toPropertyKey(descriptor.key), descriptor); } }
function _createClass(Constructor, protoProps, staticProps) { if (protoProps) _defineProperties(Constructor.prototype, protoProps); if (staticProps) _defineProperties(Constructor, staticProps); Object.defineProperty(Constructor, "prototype", { writable: false }); return Constructor; }
function _toPropertyKey(arg) { var key = _toPrimitive(arg, "string"); return _typeof(key) === "symbol" ? key : String(key); }
function _toPrimitive(input, hint) { if (_typeof(input) !== "object" || input === null) return input; var prim = input[Symbol.toPrimitive]; if (prim !== undefined) { var res = prim.call(input, hint || "default"); if (_typeof(res) !== "object") return res; throw new TypeError("@@toPrimitive must return a primitive value."); } return (hint === "string" ? String : Number)(input); }
function _classPrivateMethodInitSpec(obj, privateSet) { _checkPrivateRedeclaration(obj, privateSet); privateSet.add(obj); }
function _classPrivateFieldInitSpec(obj, privateMap, value) { _checkPrivateRedeclaration(obj, privateMap); privateMap.set(obj, value); }
function _checkPrivateRedeclaration(obj, privateCollection) { if (privateCollection.has(obj)) { throw new TypeError("Cannot initialize the same private elements twice on an object"); } }
function _classPrivateMethodGet(receiver, privateSet, fn) { if (!privateSet.has(receiver)) { throw new TypeError("attempted to get private field on non-instance"); } return fn; }
function _classPrivateFieldGet(receiver, privateMap) { var descriptor = _classExtractFieldDescriptor(receiver, privateMap, "get"); return _classApplyDescriptorGet(receiver, descriptor); }
function _classApplyDescriptorGet(receiver, descriptor) { if (descriptor.get) { return descriptor.get.call(receiver); } return descriptor.value; }
function _classPrivateFieldSet(receiver, privateMap, value) { var descriptor = _classExtractFieldDescriptor(receiver, privateMap, "set"); _classApplyDescriptorSet(receiver, descriptor, value); return value; }
function _classExtractFieldDescriptor(receiver, privateMap, action) { if (!privateMap.has(receiver)) { throw new TypeError("attempted to " + action + " private field on non-instance"); } return privateMap.get(receiver); }
function _classApplyDescriptorSet(receiver, descriptor, value) { if (descriptor.set) { descriptor.set.call(receiver, value); } else { if (!descriptor.writable) { throw new TypeError("attempted to set read only private field"); } descriptor.value = value; } }

var _entryPoint = /*#__PURE__*/new WeakMap();
var _libraries = /*#__PURE__*/new WeakMap();
var _dependencies = /*#__PURE__*/new WeakMap();
var _resolvers = /*#__PURE__*/new WeakMap();
var _breadthFirstResolveDependencies = /*#__PURE__*/new WeakSet();
var _resolveDependency = /*#__PURE__*/new WeakSet();
/**
 * Noir Dependency Resolver
 */
var NoirDependencyManager = /*#__PURE__*/function () {
  /**
   * Creates a new dependency resolver
   * @param resolvers - A list of dependency resolvers to use
   * @param entryPoint - The entry point of the project
   */
  function NoirDependencyManager() {
    var resolvers = arguments.length > 0 && arguments[0] !== undefined ? arguments[0] : [];
    var entryPoint = arguments.length > 1 ? arguments[1] : undefined;
    _classCallCheck(this, NoirDependencyManager);
    _classPrivateMethodInitSpec(this, _resolveDependency);
    _classPrivateMethodInitSpec(this, _breadthFirstResolveDependencies);
    _classPrivateFieldInitSpec(this, _entryPoint, {
      writable: true,
      value: void 0
    });
    _classPrivateFieldInitSpec(this, _libraries, {
      writable: true,
      value: new Map()
    });
    _classPrivateFieldInitSpec(this, _dependencies, {
      writable: true,
      value: new Map()
    });
    _classPrivateFieldInitSpec(this, _resolvers, {
      writable: true,
      value: void 0
    });
    _classPrivateFieldSet(this, _resolvers, resolvers);
    _classPrivateFieldSet(this, _entryPoint, entryPoint);
  }

  /**
   * Gets dependencies for the entry point
   */
  _createClass(NoirDependencyManager, [{
    key: "getEntrypointDependencies",
    value: function getEntrypointDependencies() {
      var _classPrivateFieldGet2;
      return (_classPrivateFieldGet2 = _classPrivateFieldGet(this, _dependencies).get('')) !== null && _classPrivateFieldGet2 !== void 0 ? _classPrivateFieldGet2 : [];
    }

    /**
     * A map of library dependencies
     */
  }, {
    key: "getLibraryDependencies",
    value: function getLibraryDependencies() {
      var entries = Array.from(_classPrivateFieldGet(this, _dependencies).entries());
      return Object.fromEntries(entries.filter(function (_ref) {
        var _ref2 = _slicedToArray(_ref, 1),
          name = _ref2[0];
        return name !== '';
      }));
    }

    /**
     * Resolves dependencies for a package.
     */
  }, {
    key: "resolveDependencies",
    value: function () {
      var _resolveDependencies = _asyncToGenerator( /*#__PURE__*/_regeneratorRuntime().mark(function _callee() {
        return _regeneratorRuntime().wrap(function _callee$(_context) {
          while (1) switch (_context.prev = _context.next) {
            case 0:
              _context.next = 2;
              return _classPrivateMethodGet(this, _breadthFirstResolveDependencies, _breadthFirstResolveDependencies2).call(this);
            case 2:
            case "end":
              return _context.stop();
          }
        }, _callee, this);
      }));
      function resolveDependencies() {
        return _resolveDependencies.apply(this, arguments);
      }
      return resolveDependencies;
    }()
    /**
     * Gets the version of a dependency in the dependency tree
     * @param name - Dependency name
     * @returns The dependency's version
     */
  }, {
    key: "getVersionOf",
    value: function getVersionOf(name) {
      var dep = _classPrivateFieldGet(this, _libraries).get(name);
      return dep === null || dep === void 0 ? void 0 : dep.version;
    }
  }, {
    key: "getPackageNames",
    value:
    /**
     * Gets the names of the crates in this dependency list
     */
    function getPackageNames() {
      return _toConsumableArray(_classPrivateFieldGet(this, _libraries).keys());
    }

    /**
     * Looks up a dependency
     * @param sourceId - The source being resolved
     * @returns The path to the resolved file
     */
  }, {
    key: "findFile",
    value: function findFile(sourceId) {
      var _sourceId$split$filte = sourceId.split('/').filter(function (x) {
          return x;
        }),
        _sourceId$split$filte2 = _toArray(_sourceId$split$filte),
        lib = _sourceId$split$filte2[0],
        path = _sourceId$split$filte2.slice(1);
      var dep = _classPrivateFieldGet(this, _libraries).get(lib);
      if (dep) {
        return path__WEBPACK_IMPORTED_MODULE_0__.join.apply(void 0, [dep["package"].getSrcPath()].concat(_toConsumableArray(path)));
      } else {
        return null;
      }
    }
  }]);
  return NoirDependencyManager;
}();
function _breadthFirstResolveDependencies2() {
  return _breadthFirstResolveDependencies3.apply(this, arguments);
}
function _breadthFirstResolveDependencies3() {
  _breadthFirstResolveDependencies3 = _asyncToGenerator( /*#__PURE__*/_regeneratorRuntime().mark(function _callee2() {
    var queue, _ref3, packageName, noirPackage, _i, _Object$entries, _classPrivateFieldGet4, _Object$entries$_i, name, config, _classPrivateFieldGet3, dependency;
    return _regeneratorRuntime().wrap(function _callee2$(_context2) {
      while (1) switch (_context2.prev = _context2.next) {
        case 0:
          /** Represents a package to resolve dependencies for */
          queue = [{
            packageName: '',
            noirPackage: _classPrivateFieldGet(this, _entryPoint)
          }];
        case 1:
          if (!(queue.length > 0)) {
            _context2.next = 24;
            break;
          }
          _ref3 = queue.shift(), packageName = _ref3.packageName, noirPackage = _ref3.noirPackage;
          _i = 0, _Object$entries = Object.entries(noirPackage.getDependencies());
        case 4:
          if (!(_i < _Object$entries.length)) {
            _context2.next = 22;
            break;
          }
          _Object$entries$_i = _slicedToArray(_Object$entries[_i], 2), name = _Object$entries$_i[0], config = _Object$entries$_i[1];
          if (!_classPrivateFieldGet(this, _libraries).has(name)) {
            _context2.next = 10;
            break;
          }
          console.log("skipping already resolved dependency ".concat(name));
          _classPrivateFieldGet(this, _dependencies).set(packageName, [].concat(_toConsumableArray((_classPrivateFieldGet3 = _classPrivateFieldGet(this, _dependencies).get(packageName)) !== null && _classPrivateFieldGet3 !== void 0 ? _classPrivateFieldGet3 : []), [name]));
          return _context2.abrupt("continue", 19);
        case 10:
          _context2.next = 12;
          return _classPrivateMethodGet(this, _resolveDependency, _resolveDependency2).call(this, noirPackage, config);
        case 12:
          dependency = _context2.sent;
          if (!(dependency["package"].getType() !== 'lib')) {
            _context2.next = 16;
            break;
          }
          console.log("Non-library package ".concat(name), config);
          throw new Error("Dependency ".concat(name, " is not a library"));
        case 16:
          _classPrivateFieldGet(this, _libraries).set(name, dependency);
          _classPrivateFieldGet(this, _dependencies).set(packageName, [].concat(_toConsumableArray((_classPrivateFieldGet4 = _classPrivateFieldGet(this, _dependencies).get(packageName)) !== null && _classPrivateFieldGet4 !== void 0 ? _classPrivateFieldGet4 : []), [name]));
          queue.push({
            noirPackage: dependency["package"],
            packageName: name
          });
        case 19:
          _i++;
          _context2.next = 4;
          break;
        case 22:
          _context2.next = 1;
          break;
        case 24:
        case "end":
          return _context2.stop();
      }
    }, _callee2, this);
  }));
  return _breadthFirstResolveDependencies3.apply(this, arguments);
}
function _resolveDependency2(_x, _x2) {
  return _resolveDependency3.apply(this, arguments);
}
function _resolveDependency3() {
  _resolveDependency3 = _asyncToGenerator( /*#__PURE__*/_regeneratorRuntime().mark(function _callee3(pkg, config) {
    var dependency, _iterator, _step, resolver;
    return _regeneratorRuntime().wrap(function _callee3$(_context3) {
      while (1) switch (_context3.prev = _context3.next) {
        case 0:
          dependency = null;
          _iterator = _createForOfIteratorHelper(_classPrivateFieldGet(this, _resolvers));
          _context3.prev = 2;
          _iterator.s();
        case 4:
          if ((_step = _iterator.n()).done) {
            _context3.next = 13;
            break;
          }
          resolver = _step.value;
          _context3.next = 8;
          return resolver.resolveDependency(pkg, config);
        case 8:
          dependency = _context3.sent;
          if (!dependency) {
            _context3.next = 11;
            break;
          }
          return _context3.abrupt("break", 13);
        case 11:
          _context3.next = 4;
          break;
        case 13:
          _context3.next = 18;
          break;
        case 15:
          _context3.prev = 15;
          _context3.t0 = _context3["catch"](2);
          _iterator.e(_context3.t0);
        case 18:
          _context3.prev = 18;
          _iterator.f();
          return _context3.finish(18);
        case 21:
          if (dependency) {
            _context3.next = 23;
            break;
          }
          throw new Error('Dependency not resolved');
        case 23:
          return _context3.abrupt("return", dependency);
        case 24:
        case "end":
          return _context3.stop();
      }
    }, _callee3, this, [[2, 15, 18, 21]]);
  }));
  return _resolveDependency3.apply(this, arguments);
}

/***/ }),

/***/ "./noir_wasm/src/noir/dependencies/github-dependency-resolver.ts":
/*!***********************************************************************!*\
  !*** ./noir_wasm/src/noir/dependencies/github-dependency-resolver.ts ***!
  \***********************************************************************/
/***/ ((__unused_webpack_module, __webpack_exports__, __webpack_require__) => {

"use strict";
__webpack_require__.r(__webpack_exports__);
/* harmony export */ __webpack_require__.d(__webpack_exports__, {
/* harmony export */   GithubDependencyResolver: () => (/* binding */ GithubDependencyResolver),
/* harmony export */   resolveGithubCodeArchive: () => (/* binding */ resolveGithubCodeArchive),
/* harmony export */   safeFilename: () => (/* binding */ safeFilename)
/* harmony export */ });
/* harmony import */ var path__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! path */ "path");
/* harmony import */ var path__WEBPACK_IMPORTED_MODULE_0___default = /*#__PURE__*/__webpack_require__.n(path__WEBPACK_IMPORTED_MODULE_0__);
/* harmony import */ var unzipit__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! unzipit */ "../../node_modules/unzipit/dist/unzipit.module.js");
/* harmony import */ var _package__WEBPACK_IMPORTED_MODULE_2__ = __webpack_require__(/*! ../package */ "./noir_wasm/src/noir/package.ts");
function _typeof(o) { "@babel/helpers - typeof"; return _typeof = "function" == typeof Symbol && "symbol" == typeof Symbol.iterator ? function (o) { return typeof o; } : function (o) { return o && "function" == typeof Symbol && o.constructor === Symbol && o !== Symbol.prototype ? "symbol" : typeof o; }, _typeof(o); }
function _slicedToArray(arr, i) { return _arrayWithHoles(arr) || _iterableToArrayLimit(arr, i) || _unsupportedIterableToArray(arr, i) || _nonIterableRest(); }
function _nonIterableRest() { throw new TypeError("Invalid attempt to destructure non-iterable instance.\nIn order to be iterable, non-array objects must have a [Symbol.iterator]() method."); }
function _unsupportedIterableToArray(o, minLen) { if (!o) return; if (typeof o === "string") return _arrayLikeToArray(o, minLen); var n = Object.prototype.toString.call(o).slice(8, -1); if (n === "Object" && o.constructor) n = o.constructor.name; if (n === "Map" || n === "Set") return Array.from(o); if (n === "Arguments" || /^(?:Ui|I)nt(?:8|16|32)(?:Clamped)?Array$/.test(n)) return _arrayLikeToArray(o, minLen); }
function _arrayLikeToArray(arr, len) { if (len == null || len > arr.length) len = arr.length; for (var i = 0, arr2 = new Array(len); i < len; i++) arr2[i] = arr[i]; return arr2; }
function _iterableToArrayLimit(r, l) { var t = null == r ? null : "undefined" != typeof Symbol && r[Symbol.iterator] || r["@@iterator"]; if (null != t) { var e, n, i, u, a = [], f = !0, o = !1; try { if (i = (t = t.call(r)).next, 0 === l) { if (Object(t) !== t) return; f = !1; } else for (; !(f = (e = i.call(t)).done) && (a.push(e.value), a.length !== l); f = !0); } catch (r) { o = !0, n = r; } finally { try { if (!f && null != t["return"] && (u = t["return"](), Object(u) !== u)) return; } finally { if (o) throw n; } } return a; } }
function _arrayWithHoles(arr) { if (Array.isArray(arr)) return arr; }
function _regeneratorRuntime() { "use strict"; /*! regenerator-runtime -- Copyright (c) 2014-present, Facebook, Inc. -- license (MIT): https://github.com/facebook/regenerator/blob/main/LICENSE */ _regeneratorRuntime = function _regeneratorRuntime() { return e; }; var t, e = {}, r = Object.prototype, n = r.hasOwnProperty, o = Object.defineProperty || function (t, e, r) { t[e] = r.value; }, i = "function" == typeof Symbol ? Symbol : {}, a = i.iterator || "@@iterator", c = i.asyncIterator || "@@asyncIterator", u = i.toStringTag || "@@toStringTag"; function define(t, e, r) { return Object.defineProperty(t, e, { value: r, enumerable: !0, configurable: !0, writable: !0 }), t[e]; } try { define({}, ""); } catch (t) { define = function define(t, e, r) { return t[e] = r; }; } function wrap(t, e, r, n) { var i = e && e.prototype instanceof Generator ? e : Generator, a = Object.create(i.prototype), c = new Context(n || []); return o(a, "_invoke", { value: makeInvokeMethod(t, r, c) }), a; } function tryCatch(t, e, r) { try { return { type: "normal", arg: t.call(e, r) }; } catch (t) { return { type: "throw", arg: t }; } } e.wrap = wrap; var h = "suspendedStart", l = "suspendedYield", f = "executing", s = "completed", y = {}; function Generator() {} function GeneratorFunction() {} function GeneratorFunctionPrototype() {} var p = {}; define(p, a, function () { return this; }); var d = Object.getPrototypeOf, v = d && d(d(values([]))); v && v !== r && n.call(v, a) && (p = v); var g = GeneratorFunctionPrototype.prototype = Generator.prototype = Object.create(p); function defineIteratorMethods(t) { ["next", "throw", "return"].forEach(function (e) { define(t, e, function (t) { return this._invoke(e, t); }); }); } function AsyncIterator(t, e) { function invoke(r, o, i, a) { var c = tryCatch(t[r], t, o); if ("throw" !== c.type) { var u = c.arg, h = u.value; return h && "object" == _typeof(h) && n.call(h, "__await") ? e.resolve(h.__await).then(function (t) { invoke("next", t, i, a); }, function (t) { invoke("throw", t, i, a); }) : e.resolve(h).then(function (t) { u.value = t, i(u); }, function (t) { return invoke("throw", t, i, a); }); } a(c.arg); } var r; o(this, "_invoke", { value: function value(t, n) { function callInvokeWithMethodAndArg() { return new e(function (e, r) { invoke(t, n, e, r); }); } return r = r ? r.then(callInvokeWithMethodAndArg, callInvokeWithMethodAndArg) : callInvokeWithMethodAndArg(); } }); } function makeInvokeMethod(e, r, n) { var o = h; return function (i, a) { if (o === f) throw new Error("Generator is already running"); if (o === s) { if ("throw" === i) throw a; return { value: t, done: !0 }; } for (n.method = i, n.arg = a;;) { var c = n.delegate; if (c) { var u = maybeInvokeDelegate(c, n); if (u) { if (u === y) continue; return u; } } if ("next" === n.method) n.sent = n._sent = n.arg;else if ("throw" === n.method) { if (o === h) throw o = s, n.arg; n.dispatchException(n.arg); } else "return" === n.method && n.abrupt("return", n.arg); o = f; var p = tryCatch(e, r, n); if ("normal" === p.type) { if (o = n.done ? s : l, p.arg === y) continue; return { value: p.arg, done: n.done }; } "throw" === p.type && (o = s, n.method = "throw", n.arg = p.arg); } }; } function maybeInvokeDelegate(e, r) { var n = r.method, o = e.iterator[n]; if (o === t) return r.delegate = null, "throw" === n && e.iterator["return"] && (r.method = "return", r.arg = t, maybeInvokeDelegate(e, r), "throw" === r.method) || "return" !== n && (r.method = "throw", r.arg = new TypeError("The iterator does not provide a '" + n + "' method")), y; var i = tryCatch(o, e.iterator, r.arg); if ("throw" === i.type) return r.method = "throw", r.arg = i.arg, r.delegate = null, y; var a = i.arg; return a ? a.done ? (r[e.resultName] = a.value, r.next = e.nextLoc, "return" !== r.method && (r.method = "next", r.arg = t), r.delegate = null, y) : a : (r.method = "throw", r.arg = new TypeError("iterator result is not an object"), r.delegate = null, y); } function pushTryEntry(t) { var e = { tryLoc: t[0] }; 1 in t && (e.catchLoc = t[1]), 2 in t && (e.finallyLoc = t[2], e.afterLoc = t[3]), this.tryEntries.push(e); } function resetTryEntry(t) { var e = t.completion || {}; e.type = "normal", delete e.arg, t.completion = e; } function Context(t) { this.tryEntries = [{ tryLoc: "root" }], t.forEach(pushTryEntry, this), this.reset(!0); } function values(e) { if (e || "" === e) { var r = e[a]; if (r) return r.call(e); if ("function" == typeof e.next) return e; if (!isNaN(e.length)) { var o = -1, i = function next() { for (; ++o < e.length;) if (n.call(e, o)) return next.value = e[o], next.done = !1, next; return next.value = t, next.done = !0, next; }; return i.next = i; } } throw new TypeError(_typeof(e) + " is not iterable"); } return GeneratorFunction.prototype = GeneratorFunctionPrototype, o(g, "constructor", { value: GeneratorFunctionPrototype, configurable: !0 }), o(GeneratorFunctionPrototype, "constructor", { value: GeneratorFunction, configurable: !0 }), GeneratorFunction.displayName = define(GeneratorFunctionPrototype, u, "GeneratorFunction"), e.isGeneratorFunction = function (t) { var e = "function" == typeof t && t.constructor; return !!e && (e === GeneratorFunction || "GeneratorFunction" === (e.displayName || e.name)); }, e.mark = function (t) { return Object.setPrototypeOf ? Object.setPrototypeOf(t, GeneratorFunctionPrototype) : (t.__proto__ = GeneratorFunctionPrototype, define(t, u, "GeneratorFunction")), t.prototype = Object.create(g), t; }, e.awrap = function (t) { return { __await: t }; }, defineIteratorMethods(AsyncIterator.prototype), define(AsyncIterator.prototype, c, function () { return this; }), e.AsyncIterator = AsyncIterator, e.async = function (t, r, n, o, i) { void 0 === i && (i = Promise); var a = new AsyncIterator(wrap(t, r, n, o), i); return e.isGeneratorFunction(r) ? a : a.next().then(function (t) { return t.done ? t.value : a.next(); }); }, defineIteratorMethods(g), define(g, u, "Generator"), define(g, a, function () { return this; }), define(g, "toString", function () { return "[object Generator]"; }), e.keys = function (t) { var e = Object(t), r = []; for (var n in e) r.push(n); return r.reverse(), function next() { for (; r.length;) { var t = r.pop(); if (t in e) return next.value = t, next.done = !1, next; } return next.done = !0, next; }; }, e.values = values, Context.prototype = { constructor: Context, reset: function reset(e) { if (this.prev = 0, this.next = 0, this.sent = this._sent = t, this.done = !1, this.delegate = null, this.method = "next", this.arg = t, this.tryEntries.forEach(resetTryEntry), !e) for (var r in this) "t" === r.charAt(0) && n.call(this, r) && !isNaN(+r.slice(1)) && (this[r] = t); }, stop: function stop() { this.done = !0; var t = this.tryEntries[0].completion; if ("throw" === t.type) throw t.arg; return this.rval; }, dispatchException: function dispatchException(e) { if (this.done) throw e; var r = this; function handle(n, o) { return a.type = "throw", a.arg = e, r.next = n, o && (r.method = "next", r.arg = t), !!o; } for (var o = this.tryEntries.length - 1; o >= 0; --o) { var i = this.tryEntries[o], a = i.completion; if ("root" === i.tryLoc) return handle("end"); if (i.tryLoc <= this.prev) { var c = n.call(i, "catchLoc"), u = n.call(i, "finallyLoc"); if (c && u) { if (this.prev < i.catchLoc) return handle(i.catchLoc, !0); if (this.prev < i.finallyLoc) return handle(i.finallyLoc); } else if (c) { if (this.prev < i.catchLoc) return handle(i.catchLoc, !0); } else { if (!u) throw new Error("try statement without catch or finally"); if (this.prev < i.finallyLoc) return handle(i.finallyLoc); } } } }, abrupt: function abrupt(t, e) { for (var r = this.tryEntries.length - 1; r >= 0; --r) { var o = this.tryEntries[r]; if (o.tryLoc <= this.prev && n.call(o, "finallyLoc") && this.prev < o.finallyLoc) { var i = o; break; } } i && ("break" === t || "continue" === t) && i.tryLoc <= e && e <= i.finallyLoc && (i = null); var a = i ? i.completion : {}; return a.type = t, a.arg = e, i ? (this.method = "next", this.next = i.finallyLoc, y) : this.complete(a); }, complete: function complete(t, e) { if ("throw" === t.type) throw t.arg; return "break" === t.type || "continue" === t.type ? this.next = t.arg : "return" === t.type ? (this.rval = this.arg = t.arg, this.method = "return", this.next = "end") : "normal" === t.type && e && (this.next = e), y; }, finish: function finish(t) { for (var e = this.tryEntries.length - 1; e >= 0; --e) { var r = this.tryEntries[e]; if (r.finallyLoc === t) return this.complete(r.completion, r.afterLoc), resetTryEntry(r), y; } }, "catch": function _catch(t) { for (var e = this.tryEntries.length - 1; e >= 0; --e) { var r = this.tryEntries[e]; if (r.tryLoc === t) { var n = r.completion; if ("throw" === n.type) { var o = n.arg; resetTryEntry(r); } return o; } } throw new Error("illegal catch attempt"); }, delegateYield: function delegateYield(e, r, n) { return this.delegate = { iterator: values(e), resultName: r, nextLoc: n }, "next" === this.method && (this.arg = t), y; } }, e; }
function asyncGeneratorStep(gen, resolve, reject, _next, _throw, key, arg) { try { var info = gen[key](arg); var value = info.value; } catch (error) { reject(error); return; } if (info.done) { resolve(value); } else { Promise.resolve(value).then(_next, _throw); } }
function _asyncToGenerator(fn) { return function () { var self = this, args = arguments; return new Promise(function (resolve, reject) { var gen = fn.apply(self, args); function _next(value) { asyncGeneratorStep(gen, resolve, reject, _next, _throw, "next", value); } function _throw(err) { asyncGeneratorStep(gen, resolve, reject, _next, _throw, "throw", err); } _next(undefined); }); }; }
function _classCallCheck(instance, Constructor) { if (!(instance instanceof Constructor)) { throw new TypeError("Cannot call a class as a function"); } }
function _defineProperties(target, props) { for (var i = 0; i < props.length; i++) { var descriptor = props[i]; descriptor.enumerable = descriptor.enumerable || false; descriptor.configurable = true; if ("value" in descriptor) descriptor.writable = true; Object.defineProperty(target, _toPropertyKey(descriptor.key), descriptor); } }
function _createClass(Constructor, protoProps, staticProps) { if (protoProps) _defineProperties(Constructor.prototype, protoProps); if (staticProps) _defineProperties(Constructor, staticProps); Object.defineProperty(Constructor, "prototype", { writable: false }); return Constructor; }
function _toPropertyKey(arg) { var key = _toPrimitive(arg, "string"); return _typeof(key) === "symbol" ? key : String(key); }
function _toPrimitive(input, hint) { if (_typeof(input) !== "object" || input === null) return input; var prim = input[Symbol.toPrimitive]; if (prim !== undefined) { var res = prim.call(input, hint || "default"); if (_typeof(res) !== "object") return res; throw new TypeError("@@toPrimitive must return a primitive value."); } return (hint === "string" ? String : Number)(input); }
function _classPrivateMethodInitSpec(obj, privateSet) { _checkPrivateRedeclaration(obj, privateSet); privateSet.add(obj); }
function _classPrivateFieldInitSpec(obj, privateMap, value) { _checkPrivateRedeclaration(obj, privateMap); privateMap.set(obj, value); }
function _checkPrivateRedeclaration(obj, privateCollection) { if (privateCollection.has(obj)) { throw new TypeError("Cannot initialize the same private elements twice on an object"); } }
function _classPrivateFieldGet(receiver, privateMap) { var descriptor = _classExtractFieldDescriptor(receiver, privateMap, "get"); return _classApplyDescriptorGet(receiver, descriptor); }
function _classApplyDescriptorGet(receiver, descriptor) { if (descriptor.get) { return descriptor.get.call(receiver); } return descriptor.value; }
function _classPrivateMethodGet(receiver, privateSet, fn) { if (!privateSet.has(receiver)) { throw new TypeError("attempted to get private field on non-instance"); } return fn; }
function _classPrivateFieldSet(receiver, privateMap, value) { var descriptor = _classExtractFieldDescriptor(receiver, privateMap, "set"); _classApplyDescriptorSet(receiver, descriptor, value); return value; }
function _classExtractFieldDescriptor(receiver, privateMap, action) { if (!privateMap.has(receiver)) { throw new TypeError("attempted to " + action + " private field on non-instance"); } return privateMap.get(receiver); }
function _classApplyDescriptorSet(receiver, descriptor, value) { if (descriptor.set) { descriptor.set.call(receiver, value); } else { if (!descriptor.writable) { throw new TypeError("attempted to set read only private field"); } descriptor.value = value; } }



var _fm = /*#__PURE__*/new WeakMap();
var _fetchZipFromGithub = /*#__PURE__*/new WeakSet();
var _extractZip = /*#__PURE__*/new WeakSet();
/**
 * Downloads dependencies from github
 */
var GithubDependencyResolver = /*#__PURE__*/function () {
  function GithubDependencyResolver(fm) {
    _classCallCheck(this, GithubDependencyResolver);
    _classPrivateMethodInitSpec(this, _extractZip);
    _classPrivateMethodInitSpec(this, _fetchZipFromGithub);
    _classPrivateFieldInitSpec(this, _fm, {
      writable: true,
      value: void 0
    });
    _classPrivateFieldSet(this, _fm, fm);
  }

  /**
   * Resolves a dependency from github. Returns null if URL is for a different website.
   * @param _pkg - The package to resolve the dependency for
   * @param dependency - The dependency configuration
   * @returns asd
   */
  _createClass(GithubDependencyResolver, [{
    key: "resolveDependency",
    value: function () {
      var _resolveDependency = _asyncToGenerator( /*#__PURE__*/_regeneratorRuntime().mark(function _callee(_pkg, dependency) {
        var archivePath, libPath;
        return _regeneratorRuntime().wrap(function _callee$(_context) {
          while (1) switch (_context.prev = _context.next) {
            case 0:
              if (!(!('git' in dependency) || !dependency.git.startsWith('https://github.com'))) {
                _context.next = 2;
                break;
              }
              return _context.abrupt("return", null);
            case 2:
              _context.next = 4;
              return _classPrivateMethodGet(this, _fetchZipFromGithub, _fetchZipFromGithub2).call(this, dependency);
            case 4:
              archivePath = _context.sent;
              _context.next = 7;
              return _classPrivateMethodGet(this, _extractZip, _extractZip2).call(this, dependency, archivePath);
            case 7:
              libPath = _context.sent;
              return _context.abrupt("return", {
                version: dependency.tag,
                "package": _package__WEBPACK_IMPORTED_MODULE_2__.NoirPackage.open(libPath, _classPrivateFieldGet(this, _fm))
              });
            case 9:
            case "end":
              return _context.stop();
          }
        }, _callee, this);
      }));
      function resolveDependency(_x, _x2) {
        return _resolveDependency.apply(this, arguments);
      }
      return resolveDependency;
    }()
  }]);
  return GithubDependencyResolver;
}();

/**
 * Strips the first n segments from a path
 */
function _fetchZipFromGithub2(_x3) {
  return _fetchZipFromGithub3.apply(this, arguments);
}
function _fetchZipFromGithub3() {
  _fetchZipFromGithub3 = _asyncToGenerator( /*#__PURE__*/_regeneratorRuntime().mark(function _callee2(dependency) {
    var url, localArchivePath, response, chunks, reader, _yield$reader$read, done, value, file, offset, _i, _chunks, chunk, tmpFile;
    return _regeneratorRuntime().wrap(function _callee2$(_context2) {
      while (1) switch (_context2.prev = _context2.next) {
        case 0:
          if (dependency.git.startsWith('https://github.com')) {
            _context2.next = 2;
            break;
          }
          throw new Error('Only github dependencies are supported');
        case 2:
          url = resolveGithubCodeArchive(dependency, 'zip');
          localArchivePath = (0,path__WEBPACK_IMPORTED_MODULE_0__.join)('archives', safeFilename(url.pathname)); // TODO should check signature before accepting any file
          if (!_classPrivateFieldGet(this, _fm).existsSync(localArchivePath)) {
            _context2.next = 7;
            break;
          }
          console.log('using cached archive', {
            url: url.href,
            path: localArchivePath
          });
          return _context2.abrupt("return", localArchivePath);
        case 7:
          _context2.next = 9;
          return fetch(url, {
            method: 'GET'
          });
        case 9:
          response = _context2.sent;
          if (!(!response.ok || !response.body)) {
            _context2.next = 12;
            break;
          }
          throw new Error("Failed to fetch ".concat(url, ": ").concat(response.statusText));
        case 12:
          chunks = [];
          reader = response.body.getReader();
        case 14:
          if (false) {}
          _context2.next = 17;
          return reader.read();
        case 17:
          _yield$reader$read = _context2.sent;
          done = _yield$reader$read.done;
          value = _yield$reader$read.value;
          if (!done) {
            _context2.next = 22;
            break;
          }
          return _context2.abrupt("break", 25);
        case 22:
          chunks.push(value);
          _context2.next = 14;
          break;
        case 25:
          file = new Uint8Array(chunks.reduce(function (acc, chunk) {
            return acc + chunk.length;
          }, 0));
          offset = 0;
          for (_i = 0, _chunks = chunks; _i < _chunks.length; _i++) {
            chunk = _chunks[_i];
            file.set(chunk, offset);
            offset += chunk.length;
          }
          tmpFile = localArchivePath + '.tmp';
          _classPrivateFieldGet(this, _fm).writeFileSync(tmpFile, file);
          _classPrivateFieldGet(this, _fm).mkdirSync(localArchivePath, {
            recursive: true
          });
          _classPrivateFieldGet(this, _fm).renameSync(tmpFile, localArchivePath);
          return _context2.abrupt("return", localArchivePath);
        case 33:
        case "end":
          return _context2.stop();
      }
    }, _callee2, this);
  }));
  return _fetchZipFromGithub3.apply(this, arguments);
}
function _extractZip2(_x4, _x5) {
  return _extractZip3.apply(this, arguments);
}
function _extractZip3() {
  _extractZip3 = _asyncToGenerator( /*#__PURE__*/_regeneratorRuntime().mark(function _callee3(dependency, archivePath) {
    var _dependency$tag2, _dependency$directory;
    var gitUrl, extractLocation, packagePath, _yield$unzip, entries, tmpExtractLocation, _i2, _Object$values, entry, name, path, stream, chunks, reader, _yield$reader$read2, done, value, file, offset, _i3, _chunks2, chunk;
    return _regeneratorRuntime().wrap(function _callee3$(_context3) {
      while (1) switch (_context3.prev = _context3.next) {
        case 0:
          gitUrl = new URL(dependency.git); // extract the archive to this location
          extractLocation = (0,path__WEBPACK_IMPORTED_MODULE_0__.join)('libs', safeFilename(gitUrl.pathname + '@' + ((_dependency$tag2 = dependency.tag) !== null && _dependency$tag2 !== void 0 ? _dependency$tag2 : 'HEAD'))); // where we expect to find this package after extraction
          // it might already exist if the archive got unzipped previously
          packagePath = (0,path__WEBPACK_IMPORTED_MODULE_0__.join)(extractLocation, (_dependency$directory = dependency.directory) !== null && _dependency$directory !== void 0 ? _dependency$directory : '');
          if (!_classPrivateFieldGet(this, _fm).existsSync(packagePath)) {
            _context3.next = 6;
            break;
          }
          console.log("Using existing package at ".concat(packagePath));
          return _context3.abrupt("return", packagePath);
        case 6:
          _context3.next = 8;
          return (0,unzipit__WEBPACK_IMPORTED_MODULE_1__.unzip)(_classPrivateFieldGet(this, _fm).readFileSync(archivePath));
        case 8:
          _yield$unzip = _context3.sent;
          entries = _yield$unzip.entries;
          // extract to a temporary directory, then move it to the final location
          // TODO empty the temp directory first
          tmpExtractLocation = extractLocation + '.tmp';
          _i2 = 0, _Object$values = Object.values(entries);
        case 12:
          if (!(_i2 < _Object$values.length)) {
            _context3.next = 41;
            break;
          }
          entry = _Object$values[_i2];
          if (!entry.isDirectory) {
            _context3.next = 16;
            break;
          }
          return _context3.abrupt("continue", 38);
        case 16:
          // remove the first path segment, because it'll be the archive name
          name = stripSegments(entry.name, 1);
          path = (0,path__WEBPACK_IMPORTED_MODULE_0__.join)(tmpExtractLocation, name);
          _context3.next = 20;
          return entry.blob();
        case 20:
          stream = _context3.sent.stream();
          chunks = [];
          reader = stream.getReader();
        case 23:
          if (false) {}
          _context3.next = 26;
          return reader.read();
        case 26:
          _yield$reader$read2 = _context3.sent;
          done = _yield$reader$read2.done;
          value = _yield$reader$read2.value;
          if (!done) {
            _context3.next = 31;
            break;
          }
          return _context3.abrupt("break", 34);
        case 31:
          chunks.push(value);
          _context3.next = 23;
          break;
        case 34:
          file = new Uint8Array(chunks.reduce(function (acc, chunk) {
            return acc + chunk.length;
          }, 0));
          offset = 0;
          for (_i3 = 0, _chunks2 = chunks; _i3 < _chunks2.length; _i3++) {
            chunk = _chunks2[_i3];
            file.set(chunk, offset);
            offset += chunk.length;
          }
          _classPrivateFieldGet(this, _fm).writeFileSync(path, file);
        case 38:
          _i2++;
          _context3.next = 12;
          break;
        case 41:
          _classPrivateFieldGet(this, _fm).mkdirSync(extractLocation, {
            recursive: true
          });
          _classPrivateFieldGet(this, _fm).renameSync(tmpExtractLocation, extractLocation);
          return _context3.abrupt("return", packagePath);
        case 44:
        case "end":
          return _context3.stop();
      }
    }, _callee3, this);
  }));
  return _extractZip3.apply(this, arguments);
}
function stripSegments(path, count) {
  var segments = path.split(path__WEBPACK_IMPORTED_MODULE_0__.sep).filter(Boolean);
  return segments.slice(count).join(path__WEBPACK_IMPORTED_MODULE_0__.sep);
}

/**
 * Returns a safe filename for a value
 * @param val - The value to convert
 */
function safeFilename(val) {
  if (!val) {
    throw new Error('invalid value');
  }

  // @ts-ignore
  return val.replaceAll(path__WEBPACK_IMPORTED_MODULE_0__.sep, '_').replaceAll(path__WEBPACK_IMPORTED_MODULE_0__.delimiter, '_').replace(/^_+/, '');
}

/**
 * Resolves a dependency's archive URL.
 * @param dependency - The dependency configuration
 * @returns The URL to the library archive
 */
function resolveGithubCodeArchive(dependency, format) {
  var _dependency$tag;
  var gitUrl = new URL(dependency.git);
  var _gitUrl$pathname$slic = gitUrl.pathname.slice(1).split('/'),
    _gitUrl$pathname$slic2 = _slicedToArray(_gitUrl$pathname$slic, 2),
    owner = _gitUrl$pathname$slic2[0],
    repo = _gitUrl$pathname$slic2[1];
  var ref = (_dependency$tag = dependency.tag) !== null && _dependency$tag !== void 0 ? _dependency$tag : 'HEAD';
  var extension = format === 'zip' ? 'zip' : 'tar.gz';
  if (!owner || !repo || gitUrl.hostname !== 'github.com') {
    throw new Error('Invalid Github repository URL');
  }
  return new URL("https://github.com/".concat(owner, "/").concat(repo, "/archive/").concat(ref, ".").concat(extension));
}

/***/ }),

/***/ "./noir_wasm/src/noir/dependencies/local-dependency-resolver.ts":
/*!**********************************************************************!*\
  !*** ./noir_wasm/src/noir/dependencies/local-dependency-resolver.ts ***!
  \**********************************************************************/
/***/ ((__unused_webpack_module, __webpack_exports__, __webpack_require__) => {

"use strict";
__webpack_require__.r(__webpack_exports__);
/* harmony export */ __webpack_require__.d(__webpack_exports__, {
/* harmony export */   LocalDependencyResolver: () => (/* binding */ LocalDependencyResolver)
/* harmony export */ });
/* harmony import */ var path__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! path */ "path");
/* harmony import */ var path__WEBPACK_IMPORTED_MODULE_0___default = /*#__PURE__*/__webpack_require__.n(path__WEBPACK_IMPORTED_MODULE_0__);
/* harmony import */ var _package__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! ../package */ "./noir_wasm/src/noir/package.ts");
function _typeof(o) { "@babel/helpers - typeof"; return _typeof = "function" == typeof Symbol && "symbol" == typeof Symbol.iterator ? function (o) { return typeof o; } : function (o) { return o && "function" == typeof Symbol && o.constructor === Symbol && o !== Symbol.prototype ? "symbol" : typeof o; }, _typeof(o); }
function _classCallCheck(instance, Constructor) { if (!(instance instanceof Constructor)) { throw new TypeError("Cannot call a class as a function"); } }
function _defineProperties(target, props) { for (var i = 0; i < props.length; i++) { var descriptor = props[i]; descriptor.enumerable = descriptor.enumerable || false; descriptor.configurable = true; if ("value" in descriptor) descriptor.writable = true; Object.defineProperty(target, _toPropertyKey(descriptor.key), descriptor); } }
function _createClass(Constructor, protoProps, staticProps) { if (protoProps) _defineProperties(Constructor.prototype, protoProps); if (staticProps) _defineProperties(Constructor, staticProps); Object.defineProperty(Constructor, "prototype", { writable: false }); return Constructor; }
function _toPropertyKey(arg) { var key = _toPrimitive(arg, "string"); return _typeof(key) === "symbol" ? key : String(key); }
function _toPrimitive(input, hint) { if (_typeof(input) !== "object" || input === null) return input; var prim = input[Symbol.toPrimitive]; if (prim !== undefined) { var res = prim.call(input, hint || "default"); if (_typeof(res) !== "object") return res; throw new TypeError("@@toPrimitive must return a primitive value."); } return (hint === "string" ? String : Number)(input); }
function _classPrivateFieldInitSpec(obj, privateMap, value) { _checkPrivateRedeclaration(obj, privateMap); privateMap.set(obj, value); }
function _checkPrivateRedeclaration(obj, privateCollection) { if (privateCollection.has(obj)) { throw new TypeError("Cannot initialize the same private elements twice on an object"); } }
function _classPrivateFieldGet(receiver, privateMap) { var descriptor = _classExtractFieldDescriptor(receiver, privateMap, "get"); return _classApplyDescriptorGet(receiver, descriptor); }
function _classApplyDescriptorGet(receiver, descriptor) { if (descriptor.get) { return descriptor.get.call(receiver); } return descriptor.value; }
function _classPrivateFieldSet(receiver, privateMap, value) { var descriptor = _classExtractFieldDescriptor(receiver, privateMap, "set"); _classApplyDescriptorSet(receiver, descriptor, value); return value; }
function _classExtractFieldDescriptor(receiver, privateMap, action) { if (!privateMap.has(receiver)) { throw new TypeError("attempted to " + action + " private field on non-instance"); } return privateMap.get(receiver); }
function _classApplyDescriptorSet(receiver, descriptor, value) { if (descriptor.set) { descriptor.set.call(receiver, value); } else { if (!descriptor.writable) { throw new TypeError("attempted to set read only private field"); } descriptor.value = value; } }


var _fm = /*#__PURE__*/new WeakMap();
/**
 * Resolves dependencies on-disk, relative to current package
 */
var LocalDependencyResolver = /*#__PURE__*/function () {
  function LocalDependencyResolver(fm) {
    _classCallCheck(this, LocalDependencyResolver);
    _classPrivateFieldInitSpec(this, _fm, {
      writable: true,
      value: void 0
    });
    _classPrivateFieldSet(this, _fm, fm);
  }
  _createClass(LocalDependencyResolver, [{
    key: "resolveDependency",
    value: function resolveDependency(parent, config) {
      if ('path' in config) {
        var parentPath = parent.getPackagePath();
        var dependencyPath = (0,path__WEBPACK_IMPORTED_MODULE_0__.isAbsolute)(config.path) ? config.path : (0,path__WEBPACK_IMPORTED_MODULE_0__.join)(parentPath, config.path);
        return Promise.resolve({
          // unknown version, Nargo.toml doesn't have a version field
          version: undefined,
          "package": _package__WEBPACK_IMPORTED_MODULE_1__.NoirPackage.open(dependencyPath, _classPrivateFieldGet(this, _fm))
        });
      } else {
        return Promise.resolve(null);
      }
    }
  }]);
  return LocalDependencyResolver;
}();

/***/ }),

/***/ "./noir_wasm/src/noir/noir-source-resolver.shim.ts":
/*!*********************************************************!*\
  !*** ./noir_wasm/src/noir/noir-source-resolver.shim.ts ***!
  \*********************************************************/
/***/ ((__unused_webpack_module, __webpack_exports__, __webpack_require__) => {

"use strict";
__webpack_require__.r(__webpack_exports__);
/* harmony export */ __webpack_require__.d(__webpack_exports__, {
/* harmony export */   initializeResolver: () => (/* binding */ initializeResolver)
/* harmony export */ });
// Shim module to force the use of the CJS build of source-resolver & noir_wasm
/**
 * Source resolver module
 */

// eslint-disable-next-line @typescript-eslint/no-var-requires
var sourceResolver = __webpack_require__(/*! @noir-lang/source-resolver */ "../source-resolver/lib-node/index_node.js");
var initializeResolver = sourceResolver.initializeResolver;

/***/ }),

/***/ "./noir_wasm/src/noir/noir-wasm-compiler.ts":
/*!**************************************************!*\
  !*** ./noir_wasm/src/noir/noir-wasm-compiler.ts ***!
  \**************************************************/
/***/ ((__unused_webpack_module, __webpack_exports__, __webpack_require__) => {

"use strict";
__webpack_require__.r(__webpack_exports__);
/* harmony export */ __webpack_require__.d(__webpack_exports__, {
/* harmony export */   NoirWasmContractCompiler: () => (/* binding */ NoirWasmContractCompiler)
/* harmony export */ });
/* harmony import */ var path__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! path */ "path");
/* harmony import */ var path__WEBPACK_IMPORTED_MODULE_0___default = /*#__PURE__*/__webpack_require__.n(path__WEBPACK_IMPORTED_MODULE_0__);
/* harmony import */ var _dependencies_dependency_manager__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! ./dependencies/dependency-manager */ "./noir_wasm/src/noir/dependencies/dependency-manager.ts");
/* harmony import */ var _dependencies_github_dependency_resolver__WEBPACK_IMPORTED_MODULE_2__ = __webpack_require__(/*! ./dependencies/github-dependency-resolver */ "./noir_wasm/src/noir/dependencies/github-dependency-resolver.ts");
/* harmony import */ var _dependencies_local_dependency_resolver__WEBPACK_IMPORTED_MODULE_3__ = __webpack_require__(/*! ./dependencies/local-dependency-resolver */ "./noir_wasm/src/noir/dependencies/local-dependency-resolver.ts");
/* harmony import */ var _noir_source_resolver_shim__WEBPACK_IMPORTED_MODULE_4__ = __webpack_require__(/*! ./noir-source-resolver.shim */ "./noir_wasm/src/noir/noir-source-resolver.shim.ts");
/* harmony import */ var _package__WEBPACK_IMPORTED_MODULE_5__ = __webpack_require__(/*! ./package */ "./noir_wasm/src/noir/package.ts");
/* harmony import */ var memfs__WEBPACK_IMPORTED_MODULE_6__ = __webpack_require__(/*! memfs */ "../../node_modules/memfs/lib/index.js");
/* harmony import */ var memfs__WEBPACK_IMPORTED_MODULE_6___default = /*#__PURE__*/__webpack_require__.n(memfs__WEBPACK_IMPORTED_MODULE_6__);
function _typeof(o) { "@babel/helpers - typeof"; return _typeof = "function" == typeof Symbol && "symbol" == typeof Symbol.iterator ? function (o) { return typeof o; } : function (o) { return o && "function" == typeof Symbol && o.constructor === Symbol && o !== Symbol.prototype ? "symbol" : typeof o; }, _typeof(o); }
function _createForOfIteratorHelper(o, allowArrayLike) { var it = typeof Symbol !== "undefined" && o[Symbol.iterator] || o["@@iterator"]; if (!it) { if (Array.isArray(o) || (it = _unsupportedIterableToArray(o)) || allowArrayLike && o && typeof o.length === "number") { if (it) o = it; var i = 0; var F = function F() {}; return { s: F, n: function n() { if (i >= o.length) return { done: true }; return { done: false, value: o[i++] }; }, e: function e(_e) { throw _e; }, f: F }; } throw new TypeError("Invalid attempt to iterate non-iterable instance.\nIn order to be iterable, non-array objects must have a [Symbol.iterator]() method."); } var normalCompletion = true, didErr = false, err; return { s: function s() { it = it.call(o); }, n: function n() { var step = it.next(); normalCompletion = step.done; return step; }, e: function e(_e2) { didErr = true; err = _e2; }, f: function f() { try { if (!normalCompletion && it["return"] != null) it["return"](); } finally { if (didErr) throw err; } } }; }
function _unsupportedIterableToArray(o, minLen) { if (!o) return; if (typeof o === "string") return _arrayLikeToArray(o, minLen); var n = Object.prototype.toString.call(o).slice(8, -1); if (n === "Object" && o.constructor) n = o.constructor.name; if (n === "Map" || n === "Set") return Array.from(o); if (n === "Arguments" || /^(?:Ui|I)nt(?:8|16|32)(?:Clamped)?Array$/.test(n)) return _arrayLikeToArray(o, minLen); }
function _arrayLikeToArray(arr, len) { if (len == null || len > arr.length) len = arr.length; for (var i = 0, arr2 = new Array(len); i < len; i++) arr2[i] = arr[i]; return arr2; }
function ownKeys(e, r) { var t = Object.keys(e); if (Object.getOwnPropertySymbols) { var o = Object.getOwnPropertySymbols(e); r && (o = o.filter(function (r) { return Object.getOwnPropertyDescriptor(e, r).enumerable; })), t.push.apply(t, o); } return t; }
function _objectSpread(e) { for (var r = 1; r < arguments.length; r++) { var t = null != arguments[r] ? arguments[r] : {}; r % 2 ? ownKeys(Object(t), !0).forEach(function (r) { _defineProperty(e, r, t[r]); }) : Object.getOwnPropertyDescriptors ? Object.defineProperties(e, Object.getOwnPropertyDescriptors(t)) : ownKeys(Object(t)).forEach(function (r) { Object.defineProperty(e, r, Object.getOwnPropertyDescriptor(t, r)); }); } return e; }
function _defineProperty(obj, key, value) { key = _toPropertyKey(key); if (key in obj) { Object.defineProperty(obj, key, { value: value, enumerable: true, configurable: true, writable: true }); } else { obj[key] = value; } return obj; }
function _regeneratorRuntime() { "use strict"; /*! regenerator-runtime -- Copyright (c) 2014-present, Facebook, Inc. -- license (MIT): https://github.com/facebook/regenerator/blob/main/LICENSE */ _regeneratorRuntime = function _regeneratorRuntime() { return e; }; var t, e = {}, r = Object.prototype, n = r.hasOwnProperty, o = Object.defineProperty || function (t, e, r) { t[e] = r.value; }, i = "function" == typeof Symbol ? Symbol : {}, a = i.iterator || "@@iterator", c = i.asyncIterator || "@@asyncIterator", u = i.toStringTag || "@@toStringTag"; function define(t, e, r) { return Object.defineProperty(t, e, { value: r, enumerable: !0, configurable: !0, writable: !0 }), t[e]; } try { define({}, ""); } catch (t) { define = function define(t, e, r) { return t[e] = r; }; } function wrap(t, e, r, n) { var i = e && e.prototype instanceof Generator ? e : Generator, a = Object.create(i.prototype), c = new Context(n || []); return o(a, "_invoke", { value: makeInvokeMethod(t, r, c) }), a; } function tryCatch(t, e, r) { try { return { type: "normal", arg: t.call(e, r) }; } catch (t) { return { type: "throw", arg: t }; } } e.wrap = wrap; var h = "suspendedStart", l = "suspendedYield", f = "executing", s = "completed", y = {}; function Generator() {} function GeneratorFunction() {} function GeneratorFunctionPrototype() {} var p = {}; define(p, a, function () { return this; }); var d = Object.getPrototypeOf, v = d && d(d(values([]))); v && v !== r && n.call(v, a) && (p = v); var g = GeneratorFunctionPrototype.prototype = Generator.prototype = Object.create(p); function defineIteratorMethods(t) { ["next", "throw", "return"].forEach(function (e) { define(t, e, function (t) { return this._invoke(e, t); }); }); } function AsyncIterator(t, e) { function invoke(r, o, i, a) { var c = tryCatch(t[r], t, o); if ("throw" !== c.type) { var u = c.arg, h = u.value; return h && "object" == _typeof(h) && n.call(h, "__await") ? e.resolve(h.__await).then(function (t) { invoke("next", t, i, a); }, function (t) { invoke("throw", t, i, a); }) : e.resolve(h).then(function (t) { u.value = t, i(u); }, function (t) { return invoke("throw", t, i, a); }); } a(c.arg); } var r; o(this, "_invoke", { value: function value(t, n) { function callInvokeWithMethodAndArg() { return new e(function (e, r) { invoke(t, n, e, r); }); } return r = r ? r.then(callInvokeWithMethodAndArg, callInvokeWithMethodAndArg) : callInvokeWithMethodAndArg(); } }); } function makeInvokeMethod(e, r, n) { var o = h; return function (i, a) { if (o === f) throw new Error("Generator is already running"); if (o === s) { if ("throw" === i) throw a; return { value: t, done: !0 }; } for (n.method = i, n.arg = a;;) { var c = n.delegate; if (c) { var u = maybeInvokeDelegate(c, n); if (u) { if (u === y) continue; return u; } } if ("next" === n.method) n.sent = n._sent = n.arg;else if ("throw" === n.method) { if (o === h) throw o = s, n.arg; n.dispatchException(n.arg); } else "return" === n.method && n.abrupt("return", n.arg); o = f; var p = tryCatch(e, r, n); if ("normal" === p.type) { if (o = n.done ? s : l, p.arg === y) continue; return { value: p.arg, done: n.done }; } "throw" === p.type && (o = s, n.method = "throw", n.arg = p.arg); } }; } function maybeInvokeDelegate(e, r) { var n = r.method, o = e.iterator[n]; if (o === t) return r.delegate = null, "throw" === n && e.iterator["return"] && (r.method = "return", r.arg = t, maybeInvokeDelegate(e, r), "throw" === r.method) || "return" !== n && (r.method = "throw", r.arg = new TypeError("The iterator does not provide a '" + n + "' method")), y; var i = tryCatch(o, e.iterator, r.arg); if ("throw" === i.type) return r.method = "throw", r.arg = i.arg, r.delegate = null, y; var a = i.arg; return a ? a.done ? (r[e.resultName] = a.value, r.next = e.nextLoc, "return" !== r.method && (r.method = "next", r.arg = t), r.delegate = null, y) : a : (r.method = "throw", r.arg = new TypeError("iterator result is not an object"), r.delegate = null, y); } function pushTryEntry(t) { var e = { tryLoc: t[0] }; 1 in t && (e.catchLoc = t[1]), 2 in t && (e.finallyLoc = t[2], e.afterLoc = t[3]), this.tryEntries.push(e); } function resetTryEntry(t) { var e = t.completion || {}; e.type = "normal", delete e.arg, t.completion = e; } function Context(t) { this.tryEntries = [{ tryLoc: "root" }], t.forEach(pushTryEntry, this), this.reset(!0); } function values(e) { if (e || "" === e) { var r = e[a]; if (r) return r.call(e); if ("function" == typeof e.next) return e; if (!isNaN(e.length)) { var o = -1, i = function next() { for (; ++o < e.length;) if (n.call(e, o)) return next.value = e[o], next.done = !1, next; return next.value = t, next.done = !0, next; }; return i.next = i; } } throw new TypeError(_typeof(e) + " is not iterable"); } return GeneratorFunction.prototype = GeneratorFunctionPrototype, o(g, "constructor", { value: GeneratorFunctionPrototype, configurable: !0 }), o(GeneratorFunctionPrototype, "constructor", { value: GeneratorFunction, configurable: !0 }), GeneratorFunction.displayName = define(GeneratorFunctionPrototype, u, "GeneratorFunction"), e.isGeneratorFunction = function (t) { var e = "function" == typeof t && t.constructor; return !!e && (e === GeneratorFunction || "GeneratorFunction" === (e.displayName || e.name)); }, e.mark = function (t) { return Object.setPrototypeOf ? Object.setPrototypeOf(t, GeneratorFunctionPrototype) : (t.__proto__ = GeneratorFunctionPrototype, define(t, u, "GeneratorFunction")), t.prototype = Object.create(g), t; }, e.awrap = function (t) { return { __await: t }; }, defineIteratorMethods(AsyncIterator.prototype), define(AsyncIterator.prototype, c, function () { return this; }), e.AsyncIterator = AsyncIterator, e.async = function (t, r, n, o, i) { void 0 === i && (i = Promise); var a = new AsyncIterator(wrap(t, r, n, o), i); return e.isGeneratorFunction(r) ? a : a.next().then(function (t) { return t.done ? t.value : a.next(); }); }, defineIteratorMethods(g), define(g, u, "Generator"), define(g, a, function () { return this; }), define(g, "toString", function () { return "[object Generator]"; }), e.keys = function (t) { var e = Object(t), r = []; for (var n in e) r.push(n); return r.reverse(), function next() { for (; r.length;) { var t = r.pop(); if (t in e) return next.value = t, next.done = !1, next; } return next.done = !0, next; }; }, e.values = values, Context.prototype = { constructor: Context, reset: function reset(e) { if (this.prev = 0, this.next = 0, this.sent = this._sent = t, this.done = !1, this.delegate = null, this.method = "next", this.arg = t, this.tryEntries.forEach(resetTryEntry), !e) for (var r in this) "t" === r.charAt(0) && n.call(this, r) && !isNaN(+r.slice(1)) && (this[r] = t); }, stop: function stop() { this.done = !0; var t = this.tryEntries[0].completion; if ("throw" === t.type) throw t.arg; return this.rval; }, dispatchException: function dispatchException(e) { if (this.done) throw e; var r = this; function handle(n, o) { return a.type = "throw", a.arg = e, r.next = n, o && (r.method = "next", r.arg = t), !!o; } for (var o = this.tryEntries.length - 1; o >= 0; --o) { var i = this.tryEntries[o], a = i.completion; if ("root" === i.tryLoc) return handle("end"); if (i.tryLoc <= this.prev) { var c = n.call(i, "catchLoc"), u = n.call(i, "finallyLoc"); if (c && u) { if (this.prev < i.catchLoc) return handle(i.catchLoc, !0); if (this.prev < i.finallyLoc) return handle(i.finallyLoc); } else if (c) { if (this.prev < i.catchLoc) return handle(i.catchLoc, !0); } else { if (!u) throw new Error("try statement without catch or finally"); if (this.prev < i.finallyLoc) return handle(i.finallyLoc); } } } }, abrupt: function abrupt(t, e) { for (var r = this.tryEntries.length - 1; r >= 0; --r) { var o = this.tryEntries[r]; if (o.tryLoc <= this.prev && n.call(o, "finallyLoc") && this.prev < o.finallyLoc) { var i = o; break; } } i && ("break" === t || "continue" === t) && i.tryLoc <= e && e <= i.finallyLoc && (i = null); var a = i ? i.completion : {}; return a.type = t, a.arg = e, i ? (this.method = "next", this.next = i.finallyLoc, y) : this.complete(a); }, complete: function complete(t, e) { if ("throw" === t.type) throw t.arg; return "break" === t.type || "continue" === t.type ? this.next = t.arg : "return" === t.type ? (this.rval = this.arg = t.arg, this.method = "return", this.next = "end") : "normal" === t.type && e && (this.next = e), y; }, finish: function finish(t) { for (var e = this.tryEntries.length - 1; e >= 0; --e) { var r = this.tryEntries[e]; if (r.finallyLoc === t) return this.complete(r.completion, r.afterLoc), resetTryEntry(r), y; } }, "catch": function _catch(t) { for (var e = this.tryEntries.length - 1; e >= 0; --e) { var r = this.tryEntries[e]; if (r.tryLoc === t) { var n = r.completion; if ("throw" === n.type) { var o = n.arg; resetTryEntry(r); } return o; } } throw new Error("illegal catch attempt"); }, delegateYield: function delegateYield(e, r, n) { return this.delegate = { iterator: values(e), resultName: r, nextLoc: n }, "next" === this.method && (this.arg = t), y; } }, e; }
function asyncGeneratorStep(gen, resolve, reject, _next, _throw, key, arg) { try { var info = gen[key](arg); var value = info.value; } catch (error) { reject(error); return; } if (info.done) { resolve(value); } else { Promise.resolve(value).then(_next, _throw); } }
function _asyncToGenerator(fn) { return function () { var self = this, args = arguments; return new Promise(function (resolve, reject) { var gen = fn.apply(self, args); function _next(value) { asyncGeneratorStep(gen, resolve, reject, _next, _throw, "next", value); } function _throw(err) { asyncGeneratorStep(gen, resolve, reject, _next, _throw, "throw", err); } _next(undefined); }); }; }
function _classCallCheck(instance, Constructor) { if (!(instance instanceof Constructor)) { throw new TypeError("Cannot call a class as a function"); } }
function _defineProperties(target, props) { for (var i = 0; i < props.length; i++) { var descriptor = props[i]; descriptor.enumerable = descriptor.enumerable || false; descriptor.configurable = true; if ("value" in descriptor) descriptor.writable = true; Object.defineProperty(target, _toPropertyKey(descriptor.key), descriptor); } }
function _createClass(Constructor, protoProps, staticProps) { if (protoProps) _defineProperties(Constructor.prototype, protoProps); if (staticProps) _defineProperties(Constructor, staticProps); Object.defineProperty(Constructor, "prototype", { writable: false }); return Constructor; }
function _toPropertyKey(arg) { var key = _toPrimitive(arg, "string"); return _typeof(key) === "symbol" ? key : String(key); }
function _toPrimitive(input, hint) { if (_typeof(input) !== "object" || input === null) return input; var prim = input[Symbol.toPrimitive]; if (prim !== undefined) { var res = prim.call(input, hint || "default"); if (_typeof(res) !== "object") return res; throw new TypeError("@@toPrimitive must return a primitive value."); } return (hint === "string" ? String : Number)(input); }
function _classPrivateMethodInitSpec(obj, privateSet) { _checkPrivateRedeclaration(obj, privateSet); privateSet.add(obj); }
function _classPrivateFieldInitSpec(obj, privateMap, value) { _checkPrivateRedeclaration(obj, privateMap); privateMap.set(obj, value); }
function _checkPrivateRedeclaration(obj, privateCollection) { if (privateCollection.has(obj)) { throw new TypeError("Cannot initialize the same private elements twice on an object"); } }
function _classPrivateMethodGet(receiver, privateSet, fn) { if (!privateSet.has(receiver)) { throw new TypeError("attempted to get private field on non-instance"); } return fn; }
function _classPrivateFieldGet(receiver, privateMap) { var descriptor = _classExtractFieldDescriptor(receiver, privateMap, "get"); return _classApplyDescriptorGet(receiver, descriptor); }
function _classApplyDescriptorGet(receiver, descriptor) { if (descriptor.get) { return descriptor.get.call(receiver); } return descriptor.value; }
function _classPrivateFieldSet(receiver, privateMap, value) { var descriptor = _classExtractFieldDescriptor(receiver, privateMap, "set"); _classApplyDescriptorSet(receiver, descriptor, value); return value; }
function _classExtractFieldDescriptor(receiver, privateMap, action) { if (!privateMap.has(receiver)) { throw new TypeError("attempted to " + action + " private field on non-instance"); } return privateMap.get(receiver); }
function _classApplyDescriptorSet(receiver, descriptor, value) { if (descriptor.set) { descriptor.set.call(receiver, value); } else { if (!descriptor.writable) { throw new TypeError("attempted to set read only private field"); } descriptor.value = value; } }








/**
 * Noir Package Compiler
 */
var _package = /*#__PURE__*/new WeakMap();
var _fm = /*#__PURE__*/new WeakMap();
var _dependencyManager = /*#__PURE__*/new WeakMap();
var _wasmCompiler = /*#__PURE__*/new WeakMap();
var _resolveFile = /*#__PURE__*/new WeakMap();
var _processCompileError = /*#__PURE__*/new WeakSet();
var NoirWasmContractCompiler = /*#__PURE__*/function () {
  function NoirWasmContractCompiler(entrypoint, dependencyManager, wasmCompiler) {
    var _this = this;
    _classCallCheck(this, NoirWasmContractCompiler);
    _classPrivateMethodInitSpec(this, _processCompileError);
    _classPrivateFieldInitSpec(this, _package, {
      writable: true,
      value: void 0
    });
    _classPrivateFieldInitSpec(this, _fm, {
      writable: true,
      value: void 0
    });
    _classPrivateFieldInitSpec(this, _dependencyManager, {
      writable: true,
      value: void 0
    });
    _classPrivateFieldInitSpec(this, _wasmCompiler, {
      writable: true,
      value: void 0
    });
    _classPrivateFieldInitSpec(this, _resolveFile, {
      writable: true,
      value: function value(path) {
        try {
          var libFile = _classPrivateFieldGet(_this, _dependencyManager).findFile(path);
          var data = _classPrivateFieldGet(_this, _fm).readFileSync(libFile !== null && libFile !== void 0 ? libFile : path, 'utf-8');
          return data;
        } catch (err) {
          return '';
        }
      }
    });
    _classPrivateFieldSet(this, _package, entrypoint);
    _classPrivateFieldSet(this, _fm, memfs__WEBPACK_IMPORTED_MODULE_6__.fs);
    _classPrivateFieldSet(this, _dependencyManager, dependencyManager);
    _classPrivateFieldSet(this, _wasmCompiler, wasmCompiler);
  }

  /**
   * Creates a new compiler instance.
   * @param fileManager - The file manager to use
   * @param projectPath - The path to the project
   * @param opts - Compilation options
   */
  _createClass(NoirWasmContractCompiler, [{
    key: "getResolvedAztecNrVersion",
    value:
    /**
     * Gets the version of Aztec.nr that was used compiling this contract.
     */
    function getResolvedAztecNrVersion() {
      // TODO eliminate this hardcoded library name!
      // see docs/docs/dev_docs/contracts/setup.md
      return _classPrivateFieldGet(this, _dependencyManager).getVersionOf('aztec');
    }

    /**
     * Compile EntryPoint
     */
  }, {
    key: "compile",
    value: function () {
      var _compile = _asyncToGenerator( /*#__PURE__*/_regeneratorRuntime().mark(function _callee() {
        return _regeneratorRuntime().wrap(function _callee$(_context) {
          while (1) switch (_context.prev = _context.next) {
            case 0:
              console.log("Compiling Program at ".concat(_classPrivateFieldGet(this, _package).getEntryPointPath()));
              _context.next = 3;
              return this.compileProgram();
            case 3:
              return _context.abrupt("return", _context.sent);
            case 4:
            case "end":
              return _context.stop();
          }
        }, _callee, this);
      }));
      function compile() {
        return _compile.apply(this, arguments);
      }
      return compile;
    }()
    /**
     * Compiles the Program.
     */
  }, {
    key: "compileProgram",
    value: function () {
      var _compileProgram = _asyncToGenerator( /*#__PURE__*/_regeneratorRuntime().mark(function _callee2() {
        var isContract, result;
        return _regeneratorRuntime().wrap(function _callee2$(_context2) {
          while (1) switch (_context2.prev = _context2.next) {
            case 0:
              _context2.next = 2;
              return _classPrivateFieldGet(this, _dependencyManager).resolveDependencies();
            case 2:
              console.log("Dependencies: ".concat(_classPrivateFieldGet(this, _dependencyManager).getPackageNames().join(', ')));
              (0,_noir_source_resolver_shim__WEBPACK_IMPORTED_MODULE_4__.initializeResolver)(_classPrivateFieldGet(this, _resolveFile));
              _context2.prev = 4;
              isContract = false;
              result = _classPrivateFieldGet(this, _wasmCompiler).call(this, _classPrivateFieldGet(this, _package).getEntryPointPath(), isContract, {
                /* eslint-disable camelcase */
                root_dependencies: _classPrivateFieldGet(this, _dependencyManager).getEntrypointDependencies(),
                library_dependencies: _classPrivateFieldGet(this, _dependencyManager).getLibraryDependencies()
                /* eslint-enable camelcase */
              });
              if ('program' in result) {
                _context2.next = 9;
                break;
              }
              throw new Error('No program found in compilation result');
            case 9:
              return _context2.abrupt("return", [_objectSpread({
                name: _classPrivateFieldGet(this, _package).getNoirPackageConfig()["package"].name
              }, result)]);
            case 12:
              _context2.prev = 12;
              _context2.t0 = _context2["catch"](4);
              if (_context2.t0 instanceof Error && _context2.t0.name === 'CompileError') {
                _classPrivateMethodGet(this, _processCompileError, _processCompileError2).call(this, _context2.t0);
              }
              throw _context2.t0;
            case 16:
            case "end":
              return _context2.stop();
          }
        }, _callee2, this, [[4, 12]]);
      }));
      function compileProgram() {
        return _compileProgram.apply(this, arguments);
      }
      return compileProgram;
    }()
  }], [{
    key: "new",
    value: function _new(projectPath, wasmCompiler) {
      if (!(0,path__WEBPACK_IMPORTED_MODULE_0__.isAbsolute)(projectPath)) {
        throw new Error('projectPath must be an absolute path');
      }
      var noirPackage = _package__WEBPACK_IMPORTED_MODULE_5__.NoirPackage.open(projectPath, memfs__WEBPACK_IMPORTED_MODULE_6__.fs);
      var dependencyManager = new _dependencies_dependency_manager__WEBPACK_IMPORTED_MODULE_1__.NoirDependencyManager([new _dependencies_local_dependency_resolver__WEBPACK_IMPORTED_MODULE_3__.LocalDependencyResolver(memfs__WEBPACK_IMPORTED_MODULE_6__.fs), new _dependencies_github_dependency_resolver__WEBPACK_IMPORTED_MODULE_2__.GithubDependencyResolver(memfs__WEBPACK_IMPORTED_MODULE_6__.fs)
      // TODO support actual Git repositories
      ], noirPackage);
      return new NoirWasmContractCompiler(noirPackage, dependencyManager, wasmCompiler);
    }
  }]);
  return NoirWasmContractCompiler;
}();
function _processCompileError2(err) {
  var _this2 = this;
  var _iterator = _createForOfIteratorHelper(err.diagnostics),
    _step;
  try {
    var _loop = function _loop() {
      var diag = _step.value;
      console.log("  ".concat(diag.message));
      var contents = _classPrivateFieldGet(_this2, _resolveFile).call(_this2, diag.file);
      var lines = contents.split('\n');
      var lineOffsets = lines.reduce(function (accum, _, idx) {
        if (idx === 0) {
          accum.push(0);
        } else {
          accum.push(accum[idx - 1] + lines[idx - 1].length + 1);
        }
        return accum;
      }, []);
      var _iterator2 = _createForOfIteratorHelper(diag.secondaries),
        _step2;
      try {
        var _loop2 = function _loop2() {
          var secondary = _step2.value;
          var errorLine = lineOffsets.findIndex(function (offset) {
            return offset > secondary.start;
          });
          console.log("    ".concat(diag.file, ":").concat(errorLine, ": ").concat(contents.slice(secondary.start, secondary.end)));
        };
        for (_iterator2.s(); !(_step2 = _iterator2.n()).done;) {
          _loop2();
        }
      } catch (err) {
        _iterator2.e(err);
      } finally {
        _iterator2.f();
      }
    };
    for (_iterator.s(); !(_step = _iterator.n()).done;) {
      _loop();
    }
  } catch (err) {
    _iterator.e(err);
  } finally {
    _iterator.f();
  }
}

/***/ }),

/***/ "./noir_wasm/src/noir/package.ts":
/*!***************************************!*\
  !*** ./noir_wasm/src/noir/package.ts ***!
  \***************************************/
/***/ ((__unused_webpack_module, __webpack_exports__, __webpack_require__) => {

"use strict";
__webpack_require__.r(__webpack_exports__);
/* harmony export */ __webpack_require__.d(__webpack_exports__, {
/* harmony export */   NoirPackage: () => (/* binding */ NoirPackage)
/* harmony export */ });
/* harmony import */ var _types_noir_package_config__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! ./types/noir_package_config */ "./noir_wasm/src/noir/types/noir_package_config.ts");
/* harmony import */ var _ltd_j_toml__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! @ltd/j-toml */ "../../node_modules/@ltd/j-toml/index.mjs");
/* harmony import */ var path__WEBPACK_IMPORTED_MODULE_2__ = __webpack_require__(/*! path */ "path");
/* harmony import */ var path__WEBPACK_IMPORTED_MODULE_2___default = /*#__PURE__*/__webpack_require__.n(path__WEBPACK_IMPORTED_MODULE_2__);
function _typeof(o) { "@babel/helpers - typeof"; return _typeof = "function" == typeof Symbol && "symbol" == typeof Symbol.iterator ? function (o) { return typeof o; } : function (o) { return o && "function" == typeof Symbol && o.constructor === Symbol && o !== Symbol.prototype ? "symbol" : typeof o; }, _typeof(o); }
function _classCallCheck(instance, Constructor) { if (!(instance instanceof Constructor)) { throw new TypeError("Cannot call a class as a function"); } }
function _defineProperties(target, props) { for (var i = 0; i < props.length; i++) { var descriptor = props[i]; descriptor.enumerable = descriptor.enumerable || false; descriptor.configurable = true; if ("value" in descriptor) descriptor.writable = true; Object.defineProperty(target, _toPropertyKey(descriptor.key), descriptor); } }
function _createClass(Constructor, protoProps, staticProps) { if (protoProps) _defineProperties(Constructor.prototype, protoProps); if (staticProps) _defineProperties(Constructor, staticProps); Object.defineProperty(Constructor, "prototype", { writable: false }); return Constructor; }
function _toPropertyKey(arg) { var key = _toPrimitive(arg, "string"); return _typeof(key) === "symbol" ? key : String(key); }
function _toPrimitive(input, hint) { if (_typeof(input) !== "object" || input === null) return input; var prim = input[Symbol.toPrimitive]; if (prim !== undefined) { var res = prim.call(input, hint || "default"); if (_typeof(res) !== "object") return res; throw new TypeError("@@toPrimitive must return a primitive value."); } return (hint === "string" ? String : Number)(input); }
function _classPrivateFieldInitSpec(obj, privateMap, value) { _checkPrivateRedeclaration(obj, privateMap); privateMap.set(obj, value); }
function _checkPrivateRedeclaration(obj, privateCollection) { if (privateCollection.has(obj)) { throw new TypeError("Cannot initialize the same private elements twice on an object"); } }
function _classPrivateFieldGet(receiver, privateMap) { var descriptor = _classExtractFieldDescriptor(receiver, privateMap, "get"); return _classApplyDescriptorGet(receiver, descriptor); }
function _classApplyDescriptorGet(receiver, descriptor) { if (descriptor.get) { return descriptor.get.call(receiver); } return descriptor.value; }
function _classPrivateFieldSet(receiver, privateMap, value) { var descriptor = _classExtractFieldDescriptor(receiver, privateMap, "set"); _classApplyDescriptorSet(receiver, descriptor, value); return value; }
function _classExtractFieldDescriptor(receiver, privateMap, action) { if (!privateMap.has(receiver)) { throw new TypeError("attempted to " + action + " private field on non-instance"); } return privateMap.get(receiver); }
function _classApplyDescriptorSet(receiver, descriptor, value) { if (descriptor.set) { descriptor.set.call(receiver, value); } else { if (!descriptor.writable) { throw new TypeError("attempted to set read only private field"); } descriptor.value = value; } }



var CONFIG_FILE_NAME = 'Nargo.toml';

/**
 * A Noir package.
 */
var _packagePath = /*#__PURE__*/new WeakMap();
var _srcPath = /*#__PURE__*/new WeakMap();
var _config = /*#__PURE__*/new WeakMap();
var _version = /*#__PURE__*/new WeakMap();
var NoirPackage = /*#__PURE__*/function () {
  function NoirPackage(path, srcDir, config) {
    _classCallCheck(this, NoirPackage);
    _classPrivateFieldInitSpec(this, _packagePath, {
      writable: true,
      value: void 0
    });
    _classPrivateFieldInitSpec(this, _srcPath, {
      writable: true,
      value: void 0
    });
    _classPrivateFieldInitSpec(this, _config, {
      writable: true,
      value: void 0
    });
    _classPrivateFieldInitSpec(this, _version, {
      writable: true,
      value: null
    });
    _classPrivateFieldSet(this, _packagePath, path);
    _classPrivateFieldSet(this, _srcPath, srcDir);
    _classPrivateFieldSet(this, _config, config);
  }

  /**
   * Gets this package's path.
   */
  _createClass(NoirPackage, [{
    key: "getPackagePath",
    value: function getPackagePath() {
      return _classPrivateFieldGet(this, _packagePath);
    }

    /**
     * Gets this package's Nargo.toml (NoirPackage)Config.
     */
  }, {
    key: "getNoirPackageConfig",
    value: function getNoirPackageConfig() {
      return _classPrivateFieldGet(this, _config);
    }

    /**
     * The path to the source directory.
     */
  }, {
    key: "getSrcPath",
    value: function getSrcPath() {
      return _classPrivateFieldGet(this, _srcPath);
    }

    /**
     * Gets the entrypoint path for this package.
     */
  }, {
    key: "getEntryPointPath",
    value: function getEntryPointPath() {
      var entrypoint;
      switch (this.getType()) {
        case 'lib':
          // we shouldn't need to compile `lib` type, since the .nr source is read directly
          // when the lib is used as a dependency elsewhere.
          entrypoint = 'lib.nr';
          break;
        case 'contract':
        case 'bin':
          entrypoint = 'main.nr';
          break;
        default:
          throw new Error("Unknown package type: ".concat(this.getType()));
      }

      // TODO check that `src` exists
      return (0,path__WEBPACK_IMPORTED_MODULE_2__.join)(_classPrivateFieldGet(this, _srcPath), entrypoint);
    }

    /**
     * Gets the project type
     */
  }, {
    key: "getType",
    value: function getType() {
      return _classPrivateFieldGet(this, _config)["package"].type;
    }

    /**
     * Gets this package's dependencies.
     */
  }, {
    key: "getDependencies",
    value: function getDependencies() {
      return _classPrivateFieldGet(this, _config).dependencies;
    }

    /**
     * Opens a path on the filesystem.
     * @param path - Path to the package.
     * @param fm - A file manager to use.
     * @returns The Noir package at the given location
     */
  }], [{
    key: "open",
    value: function open(path, fm) {
      var fileContents = fm.readFileSync((0,path__WEBPACK_IMPORTED_MODULE_2__.join)(path, CONFIG_FILE_NAME), 'utf-8');
      var config = (0,_types_noir_package_config__WEBPACK_IMPORTED_MODULE_0__.parseNoirPackageConfig)((0,_ltd_j_toml__WEBPACK_IMPORTED_MODULE_1__.parse)(fileContents));
      return new NoirPackage(path, (0,path__WEBPACK_IMPORTED_MODULE_2__.join)(path, 'src'), config);
    }
  }]);
  return NoirPackage;
}();

/***/ }),

/***/ "./noir_wasm/src/noir/types/noir_package_config.ts":
/*!*********************************************************!*\
  !*** ./noir_wasm/src/noir/types/noir_package_config.ts ***!
  \*********************************************************/
/***/ ((__unused_webpack_module, __webpack_exports__, __webpack_require__) => {

"use strict";
__webpack_require__.r(__webpack_exports__);
/* harmony export */ __webpack_require__.d(__webpack_exports__, {
/* harmony export */   parseNoirPackageConfig: () => (/* binding */ parseNoirPackageConfig)
/* harmony export */ });
var type = /*#__PURE__*/function (type) {
  type["lib"] = "lib";
  type["contract"] = "contract";
  type["bin"] = "bin";
  return type;
}(type || {});
/**
 * Noir package configuration.
 */
/**
 * A remote package dependency.
 */
/**
 * A local package dependency.
 */
/**
 * A package dependency.
 */
/**
 * Checks that an object is a package configuration.
 * @param config - Config to check
 */
function parseNoirPackageConfig(config) {
  return config;
}

/***/ }),

/***/ "../../node_modules/fs-monkey/lib/correctPath.js":
/*!*******************************************************!*\
  !*** ../../node_modules/fs-monkey/lib/correctPath.js ***!
  \*******************************************************/
/***/ ((__unused_webpack_module, exports) => {

"use strict";


Object.defineProperty(exports, "__esModule", ({
  value: true
}));
exports.correctPath = correctPath;
exports.unixify = unixify;
var isWin = process.platform === 'win32';
function removeTrailingSeparator(str) {
  var i = str.length - 1;
  if (i < 2) {
    return str;
  }
  while (isSeparator(str, i)) {
    i--;
  }
  return str.substr(0, i + 1);
}
function isSeparator(str, i) {
  var _char = str[i];
  return i > 0 && (_char === '/' || isWin && _char === '\\');
}
function normalizePath(str, stripTrailing) {
  if (typeof str !== 'string') {
    throw new TypeError('expected a string');
  }
  str = str.replace(/[\\\/]+/g, '/');
  if (stripTrailing !== false) {
    str = removeTrailingSeparator(str);
  }
  return str;
}
function unixify(filepath) {
  var stripTrailing = arguments.length > 1 && arguments[1] !== undefined ? arguments[1] : true;
  if (isWin) {
    filepath = normalizePath(filepath, stripTrailing);
    return filepath.replace(/^([a-zA-Z]+:|\.\/)/, '');
  }
  return filepath;
}
function correctPath(filepath) {
  return unixify(filepath.replace(/^\\\\\?\\.:\\/, '\\'));
}

/***/ }),

/***/ "../../node_modules/fs-monkey/lib/util/lists.js":
/*!******************************************************!*\
  !*** ../../node_modules/fs-monkey/lib/util/lists.js ***!
  \******************************************************/
/***/ ((__unused_webpack_module, exports) => {

"use strict";


Object.defineProperty(exports, "__esModule", ({
  value: true
}));
exports.fsSyncMethods = exports.fsProps = exports.fsAsyncMethods = void 0;
var fsProps = ['constants', 'F_OK', 'R_OK', 'W_OK', 'X_OK', 'Stats'];
exports.fsProps = fsProps;
var fsSyncMethods = ['renameSync', 'ftruncateSync', 'truncateSync', 'chownSync', 'fchownSync', 'lchownSync', 'chmodSync', 'fchmodSync', 'lchmodSync', 'statSync', 'lstatSync', 'fstatSync', 'linkSync', 'symlinkSync', 'readlinkSync', 'realpathSync', 'unlinkSync', 'rmdirSync', 'mkdirSync', 'mkdirpSync', 'readdirSync', 'closeSync', 'openSync', 'utimesSync', 'futimesSync', 'fsyncSync', 'writeSync', 'readSync', 'readFileSync', 'writeFileSync', 'appendFileSync', 'existsSync', 'accessSync', 'fdatasyncSync', 'mkdtempSync', 'copyFileSync', 'rmSync', 'createReadStream', 'createWriteStream'];
exports.fsSyncMethods = fsSyncMethods;
var fsAsyncMethods = ['rename', 'ftruncate', 'truncate', 'chown', 'fchown', 'lchown', 'chmod', 'fchmod', 'lchmod', 'stat', 'lstat', 'fstat', 'link', 'symlink', 'readlink', 'realpath', 'unlink', 'rmdir', 'mkdir', 'mkdirp', 'readdir', 'close', 'open', 'utimes', 'futimes', 'fsync', 'write', 'read', 'readFile', 'writeFile', 'appendFile', 'exists', 'access', 'fdatasync', 'mkdtemp', 'copyFile', 'rm', 'watchFile', 'unwatchFile', 'watch'];
exports.fsAsyncMethods = fsAsyncMethods;

/***/ }),

/***/ "../../node_modules/memfs/lib/Dirent.js":
/*!**********************************************!*\
  !*** ../../node_modules/memfs/lib/Dirent.js ***!
  \**********************************************/
/***/ ((__unused_webpack_module, exports, __webpack_require__) => {

"use strict";

Object.defineProperty(exports, "__esModule", ({ value: true }));
exports.Dirent = void 0;
var constants_1 = __webpack_require__(/*! ./constants */ "../../node_modules/memfs/lib/constants.js");
var encoding_1 = __webpack_require__(/*! ./encoding */ "../../node_modules/memfs/lib/encoding.js");
var S_IFMT = constants_1.constants.S_IFMT, S_IFDIR = constants_1.constants.S_IFDIR, S_IFREG = constants_1.constants.S_IFREG, S_IFBLK = constants_1.constants.S_IFBLK, S_IFCHR = constants_1.constants.S_IFCHR, S_IFLNK = constants_1.constants.S_IFLNK, S_IFIFO = constants_1.constants.S_IFIFO, S_IFSOCK = constants_1.constants.S_IFSOCK;
/**
 * A directory entry, like `fs.Dirent`.
 */
var Dirent = /** @class */ (function () {
    function Dirent() {
        this.name = '';
        this.mode = 0;
    }
    Dirent.build = function (link, encoding) {
        var dirent = new Dirent();
        var mode = link.getNode().mode;
        dirent.name = (0, encoding_1.strToEncoding)(link.getName(), encoding);
        dirent.mode = mode;
        return dirent;
    };
    Dirent.prototype._checkModeProperty = function (property) {
        return (this.mode & S_IFMT) === property;
    };
    Dirent.prototype.isDirectory = function () {
        return this._checkModeProperty(S_IFDIR);
    };
    Dirent.prototype.isFile = function () {
        return this._checkModeProperty(S_IFREG);
    };
    Dirent.prototype.isBlockDevice = function () {
        return this._checkModeProperty(S_IFBLK);
    };
    Dirent.prototype.isCharacterDevice = function () {
        return this._checkModeProperty(S_IFCHR);
    };
    Dirent.prototype.isSymbolicLink = function () {
        return this._checkModeProperty(S_IFLNK);
    };
    Dirent.prototype.isFIFO = function () {
        return this._checkModeProperty(S_IFIFO);
    };
    Dirent.prototype.isSocket = function () {
        return this._checkModeProperty(S_IFSOCK);
    };
    return Dirent;
}());
exports.Dirent = Dirent;
exports["default"] = Dirent;


/***/ }),

/***/ "../../node_modules/memfs/lib/Stats.js":
/*!*********************************************!*\
  !*** ../../node_modules/memfs/lib/Stats.js ***!
  \*********************************************/
/***/ ((__unused_webpack_module, exports, __webpack_require__) => {

"use strict";

Object.defineProperty(exports, "__esModule", ({ value: true }));
exports.Stats = void 0;
var constants_1 = __webpack_require__(/*! ./constants */ "../../node_modules/memfs/lib/constants.js");
var getBigInt_1 = __webpack_require__(/*! ./getBigInt */ "../../node_modules/memfs/lib/getBigInt.js");
var S_IFMT = constants_1.constants.S_IFMT, S_IFDIR = constants_1.constants.S_IFDIR, S_IFREG = constants_1.constants.S_IFREG, S_IFBLK = constants_1.constants.S_IFBLK, S_IFCHR = constants_1.constants.S_IFCHR, S_IFLNK = constants_1.constants.S_IFLNK, S_IFIFO = constants_1.constants.S_IFIFO, S_IFSOCK = constants_1.constants.S_IFSOCK;
/**
 * Statistics about a file/directory, like `fs.Stats`.
 */
var Stats = /** @class */ (function () {
    function Stats() {
    }
    Stats.build = function (node, bigint) {
        if (bigint === void 0) { bigint = false; }
        var stats = new Stats();
        var uid = node.uid, gid = node.gid, atime = node.atime, mtime = node.mtime, ctime = node.ctime;
        var getStatNumber = !bigint ? function (number) { return number; } : getBigInt_1.default;
        // Copy all values on Stats from Node, so that if Node values
        // change, values on Stats would still be the old ones,
        // just like in Node fs.
        stats.uid = getStatNumber(uid);
        stats.gid = getStatNumber(gid);
        stats.rdev = getStatNumber(0);
        stats.blksize = getStatNumber(4096);
        stats.ino = getStatNumber(node.ino);
        stats.size = getStatNumber(node.getSize());
        stats.blocks = getStatNumber(1);
        stats.atime = atime;
        stats.mtime = mtime;
        stats.ctime = ctime;
        stats.birthtime = ctime;
        stats.atimeMs = getStatNumber(atime.getTime());
        stats.mtimeMs = getStatNumber(mtime.getTime());
        var ctimeMs = getStatNumber(ctime.getTime());
        stats.ctimeMs = ctimeMs;
        stats.birthtimeMs = ctimeMs;
        stats.dev = getStatNumber(0);
        stats.mode = getStatNumber(node.mode);
        stats.nlink = getStatNumber(node.nlink);
        return stats;
    };
    Stats.prototype._checkModeProperty = function (property) {
        return (Number(this.mode) & S_IFMT) === property;
    };
    Stats.prototype.isDirectory = function () {
        return this._checkModeProperty(S_IFDIR);
    };
    Stats.prototype.isFile = function () {
        return this._checkModeProperty(S_IFREG);
    };
    Stats.prototype.isBlockDevice = function () {
        return this._checkModeProperty(S_IFBLK);
    };
    Stats.prototype.isCharacterDevice = function () {
        return this._checkModeProperty(S_IFCHR);
    };
    Stats.prototype.isSymbolicLink = function () {
        return this._checkModeProperty(S_IFLNK);
    };
    Stats.prototype.isFIFO = function () {
        return this._checkModeProperty(S_IFIFO);
    };
    Stats.prototype.isSocket = function () {
        return this._checkModeProperty(S_IFSOCK);
    };
    return Stats;
}());
exports.Stats = Stats;
exports["default"] = Stats;


/***/ }),

/***/ "../../node_modules/memfs/lib/constants.js":
/*!*************************************************!*\
  !*** ../../node_modules/memfs/lib/constants.js ***!
  \*************************************************/
/***/ ((__unused_webpack_module, exports) => {

"use strict";

Object.defineProperty(exports, "__esModule", ({ value: true }));
exports.constants = void 0;
exports.constants = {
    O_RDONLY: 0,
    O_WRONLY: 1,
    O_RDWR: 2,
    S_IFMT: 61440,
    S_IFREG: 32768,
    S_IFDIR: 16384,
    S_IFCHR: 8192,
    S_IFBLK: 24576,
    S_IFIFO: 4096,
    S_IFLNK: 40960,
    S_IFSOCK: 49152,
    O_CREAT: 64,
    O_EXCL: 128,
    O_NOCTTY: 256,
    O_TRUNC: 512,
    O_APPEND: 1024,
    O_DIRECTORY: 65536,
    O_NOATIME: 262144,
    O_NOFOLLOW: 131072,
    O_SYNC: 1052672,
    O_DIRECT: 16384,
    O_NONBLOCK: 2048,
    S_IRWXU: 448,
    S_IRUSR: 256,
    S_IWUSR: 128,
    S_IXUSR: 64,
    S_IRWXG: 56,
    S_IRGRP: 32,
    S_IWGRP: 16,
    S_IXGRP: 8,
    S_IRWXO: 7,
    S_IROTH: 4,
    S_IWOTH: 2,
    S_IXOTH: 1,
    F_OK: 0,
    R_OK: 4,
    W_OK: 2,
    X_OK: 1,
    UV_FS_SYMLINK_DIR: 1,
    UV_FS_SYMLINK_JUNCTION: 2,
    UV_FS_COPYFILE_EXCL: 1,
    UV_FS_COPYFILE_FICLONE: 2,
    UV_FS_COPYFILE_FICLONE_FORCE: 4,
    COPYFILE_EXCL: 1,
    COPYFILE_FICLONE: 2,
    COPYFILE_FICLONE_FORCE: 4,
};


/***/ }),

/***/ "../../node_modules/memfs/lib/encoding.js":
/*!************************************************!*\
  !*** ../../node_modules/memfs/lib/encoding.js ***!
  \************************************************/
/***/ ((__unused_webpack_module, exports, __webpack_require__) => {

"use strict";

Object.defineProperty(exports, "__esModule", ({ value: true }));
exports.strToEncoding = exports.assertEncoding = exports.ENCODING_UTF8 = void 0;
var buffer_1 = __webpack_require__(/*! ./internal/buffer */ "../../node_modules/memfs/lib/internal/buffer.js");
var errors = __webpack_require__(/*! ./internal/errors */ "../../node_modules/memfs/lib/internal/errors.js");
exports.ENCODING_UTF8 = 'utf8';
function assertEncoding(encoding) {
    if (encoding && !buffer_1.Buffer.isEncoding(encoding))
        throw new errors.TypeError('ERR_INVALID_OPT_VALUE_ENCODING', encoding);
}
exports.assertEncoding = assertEncoding;
function strToEncoding(str, encoding) {
    if (!encoding || encoding === exports.ENCODING_UTF8)
        return str; // UTF-8
    if (encoding === 'buffer')
        return new buffer_1.Buffer(str); // `buffer` encoding
    return new buffer_1.Buffer(str).toString(encoding); // Custom encoding
}
exports.strToEncoding = strToEncoding;


/***/ }),

/***/ "../../node_modules/memfs/lib/getBigInt.js":
/*!*************************************************!*\
  !*** ../../node_modules/memfs/lib/getBigInt.js ***!
  \*************************************************/
/***/ ((__unused_webpack_module, exports) => {

if (typeof BigInt === 'function') exports["default"] = BigInt;
else
  exports["default"] = function BigIntNotSupported() {
    throw new Error('BigInt is not supported in this environment.');
  };


/***/ }),

/***/ "../../node_modules/memfs/lib/index.js":
/*!*********************************************!*\
  !*** ../../node_modules/memfs/lib/index.js ***!
  \*********************************************/
/***/ (function(module, exports, __webpack_require__) {

"use strict";

var __assign = (this && this.__assign) || function () {
    __assign = Object.assign || function(t) {
        for (var s, i = 1, n = arguments.length; i < n; i++) {
            s = arguments[i];
            for (var p in s) if (Object.prototype.hasOwnProperty.call(s, p))
                t[p] = s[p];
        }
        return t;
    };
    return __assign.apply(this, arguments);
};
Object.defineProperty(exports, "__esModule", ({ value: true }));
exports.fs = exports.createFsFromVolume = exports.vol = exports.Volume = void 0;
var Stats_1 = __webpack_require__(/*! ./Stats */ "../../node_modules/memfs/lib/Stats.js");
var Dirent_1 = __webpack_require__(/*! ./Dirent */ "../../node_modules/memfs/lib/Dirent.js");
var volume_1 = __webpack_require__(/*! ./volume */ "../../node_modules/memfs/lib/volume.js");
var _a = __webpack_require__(/*! fs-monkey/lib/util/lists */ "../../node_modules/fs-monkey/lib/util/lists.js"), fsSyncMethods = _a.fsSyncMethods, fsAsyncMethods = _a.fsAsyncMethods;
var constants_1 = __webpack_require__(/*! ./constants */ "../../node_modules/memfs/lib/constants.js");
var F_OK = constants_1.constants.F_OK, R_OK = constants_1.constants.R_OK, W_OK = constants_1.constants.W_OK, X_OK = constants_1.constants.X_OK;
exports.Volume = volume_1.Volume;
// Default volume.
exports.vol = new volume_1.Volume();
function createFsFromVolume(vol) {
    var fs = { F_OK: F_OK, R_OK: R_OK, W_OK: W_OK, X_OK: X_OK, constants: constants_1.constants, Stats: Stats_1.default, Dirent: Dirent_1.default };
    // Bind FS methods.
    for (var _i = 0, fsSyncMethods_1 = fsSyncMethods; _i < fsSyncMethods_1.length; _i++) {
        var method = fsSyncMethods_1[_i];
        if (typeof vol[method] === 'function')
            fs[method] = vol[method].bind(vol);
    }
    for (var _a = 0, fsAsyncMethods_1 = fsAsyncMethods; _a < fsAsyncMethods_1.length; _a++) {
        var method = fsAsyncMethods_1[_a];
        if (typeof vol[method] === 'function')
            fs[method] = vol[method].bind(vol);
    }
    fs.StatWatcher = vol.StatWatcher;
    fs.FSWatcher = vol.FSWatcher;
    fs.WriteStream = vol.WriteStream;
    fs.ReadStream = vol.ReadStream;
    fs.promises = vol.promises;
    fs._toUnixTimestamp = volume_1.toUnixTimestamp;
    return fs;
}
exports.createFsFromVolume = createFsFromVolume;
exports.fs = createFsFromVolume(exports.vol);
module.exports = __assign(__assign({}, module.exports), exports.fs);
module.exports.semantic = true;


/***/ }),

/***/ "../../node_modules/memfs/lib/internal/buffer.js":
/*!*******************************************************!*\
  !*** ../../node_modules/memfs/lib/internal/buffer.js ***!
  \*******************************************************/
/***/ (function(__unused_webpack_module, exports, __webpack_require__) {

"use strict";

var __spreadArray = (this && this.__spreadArray) || function (to, from, pack) {
    if (pack || arguments.length === 2) for (var i = 0, l = from.length, ar; i < l; i++) {
        if (ar || !(i in from)) {
            if (!ar) ar = Array.prototype.slice.call(from, 0, i);
            ar[i] = from[i];
        }
    }
    return to.concat(ar || Array.prototype.slice.call(from));
};
Object.defineProperty(exports, "__esModule", ({ value: true }));
exports.bufferFrom = exports.bufferAllocUnsafe = exports.Buffer = void 0;
var buffer_1 = __webpack_require__(/*! buffer */ "buffer");
Object.defineProperty(exports, "Buffer", ({ enumerable: true, get: function () { return buffer_1.Buffer; } }));
function bufferV0P12Ponyfill(arg0) {
    var args = [];
    for (var _i = 1; _i < arguments.length; _i++) {
        args[_i - 1] = arguments[_i];
    }
    return new (buffer_1.Buffer.bind.apply(buffer_1.Buffer, __spreadArray([void 0, arg0], args, false)))();
}
var bufferAllocUnsafe = buffer_1.Buffer.allocUnsafe || bufferV0P12Ponyfill;
exports.bufferAllocUnsafe = bufferAllocUnsafe;
var bufferFrom = buffer_1.Buffer.from || bufferV0P12Ponyfill;
exports.bufferFrom = bufferFrom;


/***/ }),

/***/ "../../node_modules/memfs/lib/internal/errors.js":
/*!*******************************************************!*\
  !*** ../../node_modules/memfs/lib/internal/errors.js ***!
  \*******************************************************/
/***/ (function(__unused_webpack_module, exports, __webpack_require__) {

"use strict";

// The whole point behind this internal module is to allow Node.js to no
// longer be forced to treat every error message change as a semver-major
// change. The NodeError classes here all expose a `code` property whose
// value statically and permanently identifies the error. While the error
// message may change, the code should not.
var __extends = (this && this.__extends) || (function () {
    var extendStatics = function (d, b) {
        extendStatics = Object.setPrototypeOf ||
            ({ __proto__: [] } instanceof Array && function (d, b) { d.__proto__ = b; }) ||
            function (d, b) { for (var p in b) if (Object.prototype.hasOwnProperty.call(b, p)) d[p] = b[p]; };
        return extendStatics(d, b);
    };
    return function (d, b) {
        if (typeof b !== "function" && b !== null)
            throw new TypeError("Class extends value " + String(b) + " is not a constructor or null");
        extendStatics(d, b);
        function __() { this.constructor = d; }
        d.prototype = b === null ? Object.create(b) : (__.prototype = b.prototype, new __());
    };
})();
Object.defineProperty(exports, "__esModule", ({ value: true }));
exports.E = exports.AssertionError = exports.message = exports.RangeError = exports.TypeError = exports.Error = void 0;
var assert = __webpack_require__(/*! assert */ "assert");
var util = __webpack_require__(/*! util */ "util");
var kCode = typeof Symbol === 'undefined' ? '_kCode' : Symbol('code');
var messages = {}; // new Map();
function makeNodeError(Base) {
    return /** @class */ (function (_super) {
        __extends(NodeError, _super);
        function NodeError(key) {
            var args = [];
            for (var _i = 1; _i < arguments.length; _i++) {
                args[_i - 1] = arguments[_i];
            }
            var _this = _super.call(this, message(key, args)) || this;
            _this.code = key;
            _this[kCode] = key;
            _this.name = "".concat(_super.prototype.name, " [").concat(_this[kCode], "]");
            return _this;
        }
        return NodeError;
    }(Base));
}
var g = typeof globalThis !== 'undefined' ? globalThis : global;
var AssertionError = /** @class */ (function (_super) {
    __extends(AssertionError, _super);
    function AssertionError(options) {
        var _this = this;
        if (typeof options !== 'object' || options === null) {
            throw new exports.TypeError('ERR_INVALID_ARG_TYPE', 'options', 'object');
        }
        if (options.message) {
            _this = _super.call(this, options.message) || this;
        }
        else {
            _this = _super.call(this, "".concat(util.inspect(options.actual).slice(0, 128), " ") +
                "".concat(options.operator, " ").concat(util.inspect(options.expected).slice(0, 128))) || this;
        }
        _this.generatedMessage = !options.message;
        _this.name = 'AssertionError [ERR_ASSERTION]';
        _this.code = 'ERR_ASSERTION';
        _this.actual = options.actual;
        _this.expected = options.expected;
        _this.operator = options.operator;
        exports.Error.captureStackTrace(_this, options.stackStartFunction);
        return _this;
    }
    return AssertionError;
}(g.Error));
exports.AssertionError = AssertionError;
function message(key, args) {
    assert.strictEqual(typeof key, 'string');
    // const msg = messages.get(key);
    var msg = messages[key];
    assert(msg, "An invalid error message key was used: ".concat(key, "."));
    var fmt;
    if (typeof msg === 'function') {
        fmt = msg;
    }
    else {
        fmt = util.format;
        if (args === undefined || args.length === 0)
            return msg;
        args.unshift(msg);
    }
    return String(fmt.apply(null, args));
}
exports.message = message;
// Utility function for registering the error codes. Only used here. Exported
// *only* to allow for testing.
function E(sym, val) {
    messages[sym] = typeof val === 'function' ? val : String(val);
}
exports.E = E;
exports.Error = makeNodeError(g.Error);
exports.TypeError = makeNodeError(g.TypeError);
exports.RangeError = makeNodeError(g.RangeError);
// To declare an error message, use the E(sym, val) function above. The sym
// must be an upper case string. The val can be either a function or a string.
// The return value of the function must be a string.
// Examples:
// E('EXAMPLE_KEY1', 'This is the error value');
// E('EXAMPLE_KEY2', (a, b) => return `${a} ${b}`);
//
// Once an error code has been assigned, the code itself MUST NOT change and
// any given error code must never be reused to identify a different error.
//
// Any error code added here should also be added to the documentation
//
// Note: Please try to keep these in alphabetical order
E('ERR_ARG_NOT_ITERABLE', '%s must be iterable');
E('ERR_ASSERTION', '%s');
E('ERR_BUFFER_OUT_OF_BOUNDS', bufferOutOfBounds);
E('ERR_CHILD_CLOSED_BEFORE_REPLY', 'Child closed before reply received');
E('ERR_CONSOLE_WRITABLE_STREAM', 'Console expects a writable stream instance for %s');
E('ERR_CPU_USAGE', 'Unable to obtain cpu usage %s');
E('ERR_DNS_SET_SERVERS_FAILED', function (err, servers) { return "c-ares failed to set servers: \"".concat(err, "\" [").concat(servers, "]"); });
E('ERR_FALSY_VALUE_REJECTION', 'Promise was rejected with falsy value');
E('ERR_ENCODING_NOT_SUPPORTED', function (enc) { return "The \"".concat(enc, "\" encoding is not supported"); });
E('ERR_ENCODING_INVALID_ENCODED_DATA', function (enc) { return "The encoded data was not valid for encoding ".concat(enc); });
E('ERR_HTTP_HEADERS_SENT', 'Cannot render headers after they are sent to the client');
E('ERR_HTTP_INVALID_STATUS_CODE', 'Invalid status code: %s');
E('ERR_HTTP_TRAILER_INVALID', 'Trailers are invalid with this transfer encoding');
E('ERR_INDEX_OUT_OF_RANGE', 'Index out of range');
E('ERR_INVALID_ARG_TYPE', invalidArgType);
E('ERR_INVALID_ARRAY_LENGTH', function (name, len, actual) {
    assert.strictEqual(typeof actual, 'number');
    return "The array \"".concat(name, "\" (length ").concat(actual, ") must be of length ").concat(len, ".");
});
E('ERR_INVALID_BUFFER_SIZE', 'Buffer size must be a multiple of %s');
E('ERR_INVALID_CALLBACK', 'Callback must be a function');
E('ERR_INVALID_CHAR', 'Invalid character in %s');
E('ERR_INVALID_CURSOR_POS', 'Cannot set cursor row without setting its column');
E('ERR_INVALID_FD', '"fd" must be a positive integer: %s');
E('ERR_INVALID_FILE_URL_HOST', 'File URL host must be "localhost" or empty on %s');
E('ERR_INVALID_FILE_URL_PATH', 'File URL path %s');
E('ERR_INVALID_HANDLE_TYPE', 'This handle type cannot be sent');
E('ERR_INVALID_IP_ADDRESS', 'Invalid IP address: %s');
E('ERR_INVALID_OPT_VALUE', function (name, value) {
    return "The value \"".concat(String(value), "\" is invalid for option \"").concat(name, "\"");
});
E('ERR_INVALID_OPT_VALUE_ENCODING', function (value) { return "The value \"".concat(String(value), "\" is invalid for option \"encoding\""); });
E('ERR_INVALID_REPL_EVAL_CONFIG', 'Cannot specify both "breakEvalOnSigint" and "eval" for REPL');
E('ERR_INVALID_SYNC_FORK_INPUT', 'Asynchronous forks do not support Buffer, Uint8Array or string input: %s');
E('ERR_INVALID_THIS', 'Value of "this" must be of type %s');
E('ERR_INVALID_TUPLE', '%s must be an iterable %s tuple');
E('ERR_INVALID_URL', 'Invalid URL: %s');
E('ERR_INVALID_URL_SCHEME', function (expected) { return "The URL must be ".concat(oneOf(expected, 'scheme')); });
E('ERR_IPC_CHANNEL_CLOSED', 'Channel closed');
E('ERR_IPC_DISCONNECTED', 'IPC channel is already disconnected');
E('ERR_IPC_ONE_PIPE', 'Child process can have only one IPC pipe');
E('ERR_IPC_SYNC_FORK', 'IPC cannot be used with synchronous forks');
E('ERR_MISSING_ARGS', missingArgs);
E('ERR_MULTIPLE_CALLBACK', 'Callback called multiple times');
E('ERR_NAPI_CONS_FUNCTION', 'Constructor must be a function');
E('ERR_NAPI_CONS_PROTOTYPE_OBJECT', 'Constructor.prototype must be an object');
E('ERR_NO_CRYPTO', 'Node.js is not compiled with OpenSSL crypto support');
E('ERR_NO_LONGER_SUPPORTED', '%s is no longer supported');
E('ERR_PARSE_HISTORY_DATA', 'Could not parse history data in %s');
E('ERR_SOCKET_ALREADY_BOUND', 'Socket is already bound');
E('ERR_SOCKET_BAD_PORT', 'Port should be > 0 and < 65536');
E('ERR_SOCKET_BAD_TYPE', 'Bad socket type specified. Valid types are: udp4, udp6');
E('ERR_SOCKET_CANNOT_SEND', 'Unable to send data');
E('ERR_SOCKET_CLOSED', 'Socket is closed');
E('ERR_SOCKET_DGRAM_NOT_RUNNING', 'Not running');
E('ERR_STDERR_CLOSE', 'process.stderr cannot be closed');
E('ERR_STDOUT_CLOSE', 'process.stdout cannot be closed');
E('ERR_STREAM_WRAP', 'Stream has StringDecoder set or is in objectMode');
E('ERR_TLS_CERT_ALTNAME_INVALID', "Hostname/IP does not match certificate's altnames: %s");
E('ERR_TLS_DH_PARAM_SIZE', function (size) { return "DH parameter size ".concat(size, " is less than 2048"); });
E('ERR_TLS_HANDSHAKE_TIMEOUT', 'TLS handshake timeout');
E('ERR_TLS_RENEGOTIATION_FAILED', 'Failed to renegotiate');
E('ERR_TLS_REQUIRED_SERVER_NAME', '"servername" is required parameter for Server.addContext');
E('ERR_TLS_SESSION_ATTACK', 'TSL session renegotiation attack detected');
E('ERR_TRANSFORM_ALREADY_TRANSFORMING', 'Calling transform done when still transforming');
E('ERR_TRANSFORM_WITH_LENGTH_0', 'Calling transform done when writableState.length != 0');
E('ERR_UNKNOWN_ENCODING', 'Unknown encoding: %s');
E('ERR_UNKNOWN_SIGNAL', 'Unknown signal: %s');
E('ERR_UNKNOWN_STDIN_TYPE', 'Unknown stdin file type');
E('ERR_UNKNOWN_STREAM_TYPE', 'Unknown stream file type');
E('ERR_V8BREAKITERATOR', 'Full ICU data not installed. ' + 'See https://github.com/nodejs/node/wiki/Intl');
function invalidArgType(name, expected, actual) {
    assert(name, 'name is required');
    // determiner: 'must be' or 'must not be'
    var determiner;
    if (expected.includes('not ')) {
        determiner = 'must not be';
        expected = expected.split('not ')[1];
    }
    else {
        determiner = 'must be';
    }
    var msg;
    if (Array.isArray(name)) {
        var names = name.map(function (val) { return "\"".concat(val, "\""); }).join(', ');
        msg = "The ".concat(names, " arguments ").concat(determiner, " ").concat(oneOf(expected, 'type'));
    }
    else if (name.includes(' argument')) {
        // for the case like 'first argument'
        msg = "The ".concat(name, " ").concat(determiner, " ").concat(oneOf(expected, 'type'));
    }
    else {
        var type = name.includes('.') ? 'property' : 'argument';
        msg = "The \"".concat(name, "\" ").concat(type, " ").concat(determiner, " ").concat(oneOf(expected, 'type'));
    }
    // if actual value received, output it
    if (arguments.length >= 3) {
        msg += ". Received type ".concat(actual !== null ? typeof actual : 'null');
    }
    return msg;
}
function missingArgs() {
    var args = [];
    for (var _i = 0; _i < arguments.length; _i++) {
        args[_i] = arguments[_i];
    }
    assert(args.length > 0, 'At least one arg needs to be specified');
    var msg = 'The ';
    var len = args.length;
    args = args.map(function (a) { return "\"".concat(a, "\""); });
    switch (len) {
        case 1:
            msg += "".concat(args[0], " argument");
            break;
        case 2:
            msg += "".concat(args[0], " and ").concat(args[1], " arguments");
            break;
        default:
            msg += args.slice(0, len - 1).join(', ');
            msg += ", and ".concat(args[len - 1], " arguments");
            break;
    }
    return "".concat(msg, " must be specified");
}
function oneOf(expected, thing) {
    assert(expected, 'expected is required');
    assert(typeof thing === 'string', 'thing is required');
    if (Array.isArray(expected)) {
        var len = expected.length;
        assert(len > 0, 'At least one expected value needs to be specified');
        // tslint:disable-next-line
        expected = expected.map(function (i) { return String(i); });
        if (len > 2) {
            return "one of ".concat(thing, " ").concat(expected.slice(0, len - 1).join(', '), ", or ") + expected[len - 1];
        }
        else if (len === 2) {
            return "one of ".concat(thing, " ").concat(expected[0], " or ").concat(expected[1]);
        }
        else {
            return "of ".concat(thing, " ").concat(expected[0]);
        }
    }
    else {
        return "of ".concat(thing, " ").concat(String(expected));
    }
}
function bufferOutOfBounds(name, isWriting) {
    if (isWriting) {
        return 'Attempt to write outside buffer bounds';
    }
    else {
        return "\"".concat(name, "\" is outside of buffer bounds");
    }
}


/***/ }),

/***/ "../../node_modules/memfs/lib/node.js":
/*!********************************************!*\
  !*** ../../node_modules/memfs/lib/node.js ***!
  \********************************************/
/***/ (function(__unused_webpack_module, exports, __webpack_require__) {

"use strict";

var __extends = (this && this.__extends) || (function () {
    var extendStatics = function (d, b) {
        extendStatics = Object.setPrototypeOf ||
            ({ __proto__: [] } instanceof Array && function (d, b) { d.__proto__ = b; }) ||
            function (d, b) { for (var p in b) if (Object.prototype.hasOwnProperty.call(b, p)) d[p] = b[p]; };
        return extendStatics(d, b);
    };
    return function (d, b) {
        if (typeof b !== "function" && b !== null)
            throw new TypeError("Class extends value " + String(b) + " is not a constructor or null");
        extendStatics(d, b);
        function __() { this.constructor = d; }
        d.prototype = b === null ? Object.create(b) : (__.prototype = b.prototype, new __());
    };
})();
Object.defineProperty(exports, "__esModule", ({ value: true }));
exports.File = exports.Link = exports.Node = exports.SEP = void 0;
var process_1 = __webpack_require__(/*! ./process */ "../../node_modules/memfs/lib/process.js");
var buffer_1 = __webpack_require__(/*! ./internal/buffer */ "../../node_modules/memfs/lib/internal/buffer.js");
var constants_1 = __webpack_require__(/*! ./constants */ "../../node_modules/memfs/lib/constants.js");
var events_1 = __webpack_require__(/*! events */ "events");
var Stats_1 = __webpack_require__(/*! ./Stats */ "../../node_modules/memfs/lib/Stats.js");
var S_IFMT = constants_1.constants.S_IFMT, S_IFDIR = constants_1.constants.S_IFDIR, S_IFREG = constants_1.constants.S_IFREG, S_IFLNK = constants_1.constants.S_IFLNK, O_APPEND = constants_1.constants.O_APPEND;
var getuid = function () { var _a, _b; return (_b = (_a = process_1.default.getuid) === null || _a === void 0 ? void 0 : _a.call(process_1.default)) !== null && _b !== void 0 ? _b : 0; };
var getgid = function () { var _a, _b; return (_b = (_a = process_1.default.getgid) === null || _a === void 0 ? void 0 : _a.call(process_1.default)) !== null && _b !== void 0 ? _b : 0; };
exports.SEP = '/';
/**
 * Node in a file system (like i-node, v-node).
 */
var Node = /** @class */ (function (_super) {
    __extends(Node, _super);
    function Node(ino, perm) {
        if (perm === void 0) { perm = 438; }
        var _this = _super.call(this) || this;
        // User ID and group ID.
        _this._uid = getuid();
        _this._gid = getgid();
        _this._atime = new Date();
        _this._mtime = new Date();
        _this._ctime = new Date();
        _this._perm = 438; // Permissions `chmod`, `fchmod`
        _this.mode = S_IFREG; // S_IFDIR, S_IFREG, etc.. (file by default?)
        // Number of hard links pointing at this Node.
        _this._nlink = 1;
        _this._perm = perm;
        _this.mode |= perm;
        _this.ino = ino;
        return _this;
    }
    Object.defineProperty(Node.prototype, "ctime", {
        get: function () {
            return this._ctime;
        },
        set: function (ctime) {
            this._ctime = ctime;
        },
        enumerable: false,
        configurable: true
    });
    Object.defineProperty(Node.prototype, "uid", {
        get: function () {
            return this._uid;
        },
        set: function (uid) {
            this._uid = uid;
            this.ctime = new Date();
        },
        enumerable: false,
        configurable: true
    });
    Object.defineProperty(Node.prototype, "gid", {
        get: function () {
            return this._gid;
        },
        set: function (gid) {
            this._gid = gid;
            this.ctime = new Date();
        },
        enumerable: false,
        configurable: true
    });
    Object.defineProperty(Node.prototype, "atime", {
        get: function () {
            return this._atime;
        },
        set: function (atime) {
            this._atime = atime;
            this.ctime = new Date();
        },
        enumerable: false,
        configurable: true
    });
    Object.defineProperty(Node.prototype, "mtime", {
        get: function () {
            return this._mtime;
        },
        set: function (mtime) {
            this._mtime = mtime;
            this.ctime = new Date();
        },
        enumerable: false,
        configurable: true
    });
    Object.defineProperty(Node.prototype, "perm", {
        get: function () {
            return this._perm;
        },
        set: function (perm) {
            this._perm = perm;
            this.ctime = new Date();
        },
        enumerable: false,
        configurable: true
    });
    Object.defineProperty(Node.prototype, "nlink", {
        get: function () {
            return this._nlink;
        },
        set: function (nlink) {
            this._nlink = nlink;
            this.ctime = new Date();
        },
        enumerable: false,
        configurable: true
    });
    Node.prototype.getString = function (encoding) {
        if (encoding === void 0) { encoding = 'utf8'; }
        this.atime = new Date();
        return this.getBuffer().toString(encoding);
    };
    Node.prototype.setString = function (str) {
        // this.setBuffer(bufferFrom(str, 'utf8'));
        this.buf = (0, buffer_1.bufferFrom)(str, 'utf8');
        this.touch();
    };
    Node.prototype.getBuffer = function () {
        this.atime = new Date();
        if (!this.buf)
            this.setBuffer((0, buffer_1.bufferAllocUnsafe)(0));
        return (0, buffer_1.bufferFrom)(this.buf); // Return a copy.
    };
    Node.prototype.setBuffer = function (buf) {
        this.buf = (0, buffer_1.bufferFrom)(buf); // Creates a copy of data.
        this.touch();
    };
    Node.prototype.getSize = function () {
        return this.buf ? this.buf.length : 0;
    };
    Node.prototype.setModeProperty = function (property) {
        this.mode = (this.mode & ~S_IFMT) | property;
    };
    Node.prototype.setIsFile = function () {
        this.setModeProperty(S_IFREG);
    };
    Node.prototype.setIsDirectory = function () {
        this.setModeProperty(S_IFDIR);
    };
    Node.prototype.setIsSymlink = function () {
        this.setModeProperty(S_IFLNK);
    };
    Node.prototype.isFile = function () {
        return (this.mode & S_IFMT) === S_IFREG;
    };
    Node.prototype.isDirectory = function () {
        return (this.mode & S_IFMT) === S_IFDIR;
    };
    Node.prototype.isSymlink = function () {
        // return !!this.symlink;
        return (this.mode & S_IFMT) === S_IFLNK;
    };
    Node.prototype.makeSymlink = function (steps) {
        this.symlink = steps;
        this.setIsSymlink();
    };
    Node.prototype.write = function (buf, off, len, pos) {
        if (off === void 0) { off = 0; }
        if (len === void 0) { len = buf.length; }
        if (pos === void 0) { pos = 0; }
        if (!this.buf)
            this.buf = (0, buffer_1.bufferAllocUnsafe)(0);
        if (pos + len > this.buf.length) {
            var newBuf = (0, buffer_1.bufferAllocUnsafe)(pos + len);
            this.buf.copy(newBuf, 0, 0, this.buf.length);
            this.buf = newBuf;
        }
        buf.copy(this.buf, pos, off, off + len);
        this.touch();
        return len;
    };
    // Returns the number of bytes read.
    Node.prototype.read = function (buf, off, len, pos) {
        if (off === void 0) { off = 0; }
        if (len === void 0) { len = buf.byteLength; }
        if (pos === void 0) { pos = 0; }
        this.atime = new Date();
        if (!this.buf)
            this.buf = (0, buffer_1.bufferAllocUnsafe)(0);
        var actualLen = len;
        if (actualLen > buf.byteLength) {
            actualLen = buf.byteLength;
        }
        if (actualLen + pos > this.buf.length) {
            actualLen = this.buf.length - pos;
        }
        this.buf.copy(buf, off, pos, pos + actualLen);
        return actualLen;
    };
    Node.prototype.truncate = function (len) {
        if (len === void 0) { len = 0; }
        if (!len)
            this.buf = (0, buffer_1.bufferAllocUnsafe)(0);
        else {
            if (!this.buf)
                this.buf = (0, buffer_1.bufferAllocUnsafe)(0);
            if (len <= this.buf.length) {
                this.buf = this.buf.slice(0, len);
            }
            else {
                var buf = (0, buffer_1.bufferAllocUnsafe)(len);
                this.buf.copy(buf);
                buf.fill(0, this.buf.length);
                this.buf = buf;
            }
        }
        this.touch();
    };
    Node.prototype.chmod = function (perm) {
        this.perm = perm;
        this.mode = (this.mode & ~511) | perm;
        this.touch();
    };
    Node.prototype.chown = function (uid, gid) {
        this.uid = uid;
        this.gid = gid;
        this.touch();
    };
    Node.prototype.touch = function () {
        this.mtime = new Date();
        this.emit('change', this);
    };
    Node.prototype.canRead = function (uid, gid) {
        if (uid === void 0) { uid = getuid(); }
        if (gid === void 0) { gid = getgid(); }
        if (this.perm & 4 /* S.IROTH */) {
            return true;
        }
        if (gid === this.gid) {
            if (this.perm & 32 /* S.IRGRP */) {
                return true;
            }
        }
        if (uid === this.uid) {
            if (this.perm & 256 /* S.IRUSR */) {
                return true;
            }
        }
        return false;
    };
    Node.prototype.canWrite = function (uid, gid) {
        if (uid === void 0) { uid = getuid(); }
        if (gid === void 0) { gid = getgid(); }
        if (this.perm & 2 /* S.IWOTH */) {
            return true;
        }
        if (gid === this.gid) {
            if (this.perm & 16 /* S.IWGRP */) {
                return true;
            }
        }
        if (uid === this.uid) {
            if (this.perm & 128 /* S.IWUSR */) {
                return true;
            }
        }
        return false;
    };
    Node.prototype.del = function () {
        this.emit('delete', this);
    };
    Node.prototype.toJSON = function () {
        return {
            ino: this.ino,
            uid: this.uid,
            gid: this.gid,
            atime: this.atime.getTime(),
            mtime: this.mtime.getTime(),
            ctime: this.ctime.getTime(),
            perm: this.perm,
            mode: this.mode,
            nlink: this.nlink,
            symlink: this.symlink,
            data: this.getString(),
        };
    };
    return Node;
}(events_1.EventEmitter));
exports.Node = Node;
/**
 * Represents a hard link that points to an i-node `node`.
 */
var Link = /** @class */ (function (_super) {
    __extends(Link, _super);
    function Link(vol, parent, name) {
        var _this = _super.call(this) || this;
        _this.children = {};
        // Path to this node as Array: ['usr', 'bin', 'node'].
        _this._steps = [];
        // "i-node" number of the node.
        _this.ino = 0;
        // Number of children.
        _this.length = 0;
        _this.vol = vol;
        _this.parent = parent;
        _this.name = name;
        _this.syncSteps();
        return _this;
    }
    Object.defineProperty(Link.prototype, "steps", {
        get: function () {
            return this._steps;
        },
        // Recursively sync children steps, e.g. in case of dir rename
        set: function (val) {
            this._steps = val;
            for (var _i = 0, _a = Object.entries(this.children); _i < _a.length; _i++) {
                var _b = _a[_i], child = _b[0], link = _b[1];
                if (child === '.' || child === '..') {
                    continue;
                }
                link === null || link === void 0 ? void 0 : link.syncSteps();
            }
        },
        enumerable: false,
        configurable: true
    });
    Link.prototype.setNode = function (node) {
        this.node = node;
        this.ino = node.ino;
    };
    Link.prototype.getNode = function () {
        return this.node;
    };
    Link.prototype.createChild = function (name, node) {
        if (node === void 0) { node = this.vol.createNode(); }
        var link = new Link(this.vol, this, name);
        link.setNode(node);
        if (node.isDirectory()) {
            link.children['.'] = link;
            link.getNode().nlink++;
        }
        this.setChild(name, link);
        return link;
    };
    Link.prototype.setChild = function (name, link) {
        if (link === void 0) { link = new Link(this.vol, this, name); }
        this.children[name] = link;
        link.parent = this;
        this.length++;
        var node = link.getNode();
        if (node.isDirectory()) {
            link.children['..'] = this;
            this.getNode().nlink++;
        }
        this.getNode().mtime = new Date();
        this.emit('child:add', link, this);
        return link;
    };
    Link.prototype.deleteChild = function (link) {
        var node = link.getNode();
        if (node.isDirectory()) {
            delete link.children['..'];
            this.getNode().nlink--;
        }
        delete this.children[link.getName()];
        this.length--;
        this.getNode().mtime = new Date();
        this.emit('child:delete', link, this);
    };
    Link.prototype.getChild = function (name) {
        this.getNode().mtime = new Date();
        if (Object.hasOwnProperty.call(this.children, name)) {
            return this.children[name];
        }
    };
    Link.prototype.getPath = function () {
        return this.steps.join(exports.SEP);
    };
    Link.prototype.getName = function () {
        return this.steps[this.steps.length - 1];
    };
    // del() {
    //     const parent = this.parent;
    //     if(parent) {
    //         parent.deleteChild(link);
    //     }
    //     this.parent = null;
    //     this.vol = null;
    // }
    /**
     * Walk the tree path and return the `Link` at that location, if any.
     * @param steps {string[]} Desired location.
     * @param stop {number} Max steps to go into.
     * @param i {number} Current step in the `steps` array.
     *
     * @return {Link|null}
     */
    Link.prototype.walk = function (steps, stop, i) {
        if (stop === void 0) { stop = steps.length; }
        if (i === void 0) { i = 0; }
        if (i >= steps.length)
            return this;
        if (i >= stop)
            return this;
        var step = steps[i];
        var link = this.getChild(step);
        if (!link)
            return null;
        return link.walk(steps, stop, i + 1);
    };
    Link.prototype.toJSON = function () {
        return {
            steps: this.steps,
            ino: this.ino,
            children: Object.keys(this.children),
        };
    };
    Link.prototype.syncSteps = function () {
        this.steps = this.parent ? this.parent.steps.concat([this.name]) : [this.name];
    };
    return Link;
}(events_1.EventEmitter));
exports.Link = Link;
/**
 * Represents an open file (file descriptor) that points to a `Link` (Hard-link) and a `Node`.
 */
var File = /** @class */ (function () {
    /**
     * Open a Link-Node pair. `node` is provided separately as that might be a different node
     * rather the one `link` points to, because it might be a symlink.
     * @param link
     * @param node
     * @param flags
     * @param fd
     */
    function File(link, node, flags, fd) {
        /**
         * A cursor/offset position in a file, where data will be written on write.
         * User can "seek" this position.
         */
        this.position = 0;
        this.link = link;
        this.node = node;
        this.flags = flags;
        this.fd = fd;
    }
    File.prototype.getString = function (encoding) {
        if (encoding === void 0) { encoding = 'utf8'; }
        return this.node.getString();
    };
    File.prototype.setString = function (str) {
        this.node.setString(str);
    };
    File.prototype.getBuffer = function () {
        return this.node.getBuffer();
    };
    File.prototype.setBuffer = function (buf) {
        this.node.setBuffer(buf);
    };
    File.prototype.getSize = function () {
        return this.node.getSize();
    };
    File.prototype.truncate = function (len) {
        this.node.truncate(len);
    };
    File.prototype.seekTo = function (position) {
        this.position = position;
    };
    File.prototype.stats = function () {
        return Stats_1.default.build(this.node);
    };
    File.prototype.write = function (buf, offset, length, position) {
        if (offset === void 0) { offset = 0; }
        if (length === void 0) { length = buf.length; }
        if (typeof position !== 'number')
            position = this.position;
        if (this.flags & O_APPEND)
            position = this.getSize();
        var bytes = this.node.write(buf, offset, length, position);
        this.position = position + bytes;
        return bytes;
    };
    File.prototype.read = function (buf, offset, length, position) {
        if (offset === void 0) { offset = 0; }
        if (length === void 0) { length = buf.byteLength; }
        if (typeof position !== 'number')
            position = this.position;
        var bytes = this.node.read(buf, offset, length, position);
        this.position = position + bytes;
        return bytes;
    };
    File.prototype.chmod = function (perm) {
        this.node.chmod(perm);
    };
    File.prototype.chown = function (uid, gid) {
        this.node.chown(uid, gid);
    };
    return File;
}());
exports.File = File;


/***/ }),

/***/ "../../node_modules/memfs/lib/process.js":
/*!***********************************************!*\
  !*** ../../node_modules/memfs/lib/process.js ***!
  \***********************************************/
/***/ ((__unused_webpack_module, exports, __webpack_require__) => {

"use strict";

// Here we mock the global `process` variable in case we are not in Node's environment.
Object.defineProperty(exports, "__esModule", ({ value: true }));
exports.createProcess = void 0;
/**
 * Looks to return a `process` object, if one is available.
 *
 * The global `process` is returned if defined;
 * otherwise `require('process')` is attempted.
 *
 * If that fails, `undefined` is returned.
 *
 * @return {IProcess | undefined}
 */
var maybeReturnProcess = function () {
    if (typeof process !== 'undefined') {
        return process;
    }
    try {
        return __webpack_require__(/*! process */ "process");
    }
    catch (_a) {
        return undefined;
    }
};
function createProcess() {
    var p = maybeReturnProcess() || {};
    if (!p.cwd)
        p.cwd = function () { return '/'; };
    if (!p.nextTick)
        p.nextTick = (__webpack_require__(/*! ./setImmediate */ "../../node_modules/memfs/lib/setImmediate.js")["default"]);
    if (!p.emitWarning)
        p.emitWarning = function (message, type) {
            // tslint:disable-next-line:no-console
            console.warn("".concat(type).concat(type ? ': ' : '').concat(message));
        };
    if (!p.env)
        p.env = {};
    return p;
}
exports.createProcess = createProcess;
exports["default"] = createProcess();


/***/ }),

/***/ "../../node_modules/memfs/lib/promises.js":
/*!************************************************!*\
  !*** ../../node_modules/memfs/lib/promises.js ***!
  \************************************************/
/***/ (function(__unused_webpack_module, exports) {

"use strict";

var __spreadArray = (this && this.__spreadArray) || function (to, from, pack) {
    if (pack || arguments.length === 2) for (var i = 0, l = from.length, ar; i < l; i++) {
        if (ar || !(i in from)) {
            if (!ar) ar = Array.prototype.slice.call(from, 0, i);
            ar[i] = from[i];
        }
    }
    return to.concat(ar || Array.prototype.slice.call(from));
};
Object.defineProperty(exports, "__esModule", ({ value: true }));
exports.FileHandle = void 0;
function promisify(vol, fn, getResult) {
    if (getResult === void 0) { getResult = function (input) { return input; }; }
    return function () {
        var args = [];
        for (var _i = 0; _i < arguments.length; _i++) {
            args[_i] = arguments[_i];
        }
        return new Promise(function (resolve, reject) {
            vol[fn].bind(vol).apply(void 0, __spreadArray(__spreadArray([], args, false), [function (error, result) {
                    if (error)
                        return reject(error);
                    return resolve(getResult(result));
                }], false));
        });
    };
}
var FileHandle = /** @class */ (function () {
    function FileHandle(vol, fd) {
        this.vol = vol;
        this.fd = fd;
    }
    FileHandle.prototype.appendFile = function (data, options) {
        return promisify(this.vol, 'appendFile')(this.fd, data, options);
    };
    FileHandle.prototype.chmod = function (mode) {
        return promisify(this.vol, 'fchmod')(this.fd, mode);
    };
    FileHandle.prototype.chown = function (uid, gid) {
        return promisify(this.vol, 'fchown')(this.fd, uid, gid);
    };
    FileHandle.prototype.close = function () {
        return promisify(this.vol, 'close')(this.fd);
    };
    FileHandle.prototype.datasync = function () {
        return promisify(this.vol, 'fdatasync')(this.fd);
    };
    FileHandle.prototype.read = function (buffer, offset, length, position) {
        return promisify(this.vol, 'read', function (bytesRead) { return ({ bytesRead: bytesRead, buffer: buffer }); })(this.fd, buffer, offset, length, position);
    };
    FileHandle.prototype.readFile = function (options) {
        return promisify(this.vol, 'readFile')(this.fd, options);
    };
    FileHandle.prototype.stat = function (options) {
        return promisify(this.vol, 'fstat')(this.fd, options);
    };
    FileHandle.prototype.sync = function () {
        return promisify(this.vol, 'fsync')(this.fd);
    };
    FileHandle.prototype.truncate = function (len) {
        return promisify(this.vol, 'ftruncate')(this.fd, len);
    };
    FileHandle.prototype.utimes = function (atime, mtime) {
        return promisify(this.vol, 'futimes')(this.fd, atime, mtime);
    };
    FileHandle.prototype.write = function (buffer, offset, length, position) {
        return promisify(this.vol, 'write', function (bytesWritten) { return ({ bytesWritten: bytesWritten, buffer: buffer }); })(this.fd, buffer, offset, length, position);
    };
    FileHandle.prototype.writeFile = function (data, options) {
        return promisify(this.vol, 'writeFile')(this.fd, data, options);
    };
    return FileHandle;
}());
exports.FileHandle = FileHandle;
function createPromisesApi(vol) {
    if (typeof Promise === 'undefined')
        return null;
    return {
        FileHandle: FileHandle,
        access: function (path, mode) {
            return promisify(vol, 'access')(path, mode);
        },
        appendFile: function (path, data, options) {
            return promisify(vol, 'appendFile')(path instanceof FileHandle ? path.fd : path, data, options);
        },
        chmod: function (path, mode) {
            return promisify(vol, 'chmod')(path, mode);
        },
        chown: function (path, uid, gid) {
            return promisify(vol, 'chown')(path, uid, gid);
        },
        copyFile: function (src, dest, flags) {
            return promisify(vol, 'copyFile')(src, dest, flags);
        },
        lchmod: function (path, mode) {
            return promisify(vol, 'lchmod')(path, mode);
        },
        lchown: function (path, uid, gid) {
            return promisify(vol, 'lchown')(path, uid, gid);
        },
        link: function (existingPath, newPath) {
            return promisify(vol, 'link')(existingPath, newPath);
        },
        lstat: function (path, options) {
            return promisify(vol, 'lstat')(path, options);
        },
        mkdir: function (path, options) {
            return promisify(vol, 'mkdir')(path, options);
        },
        mkdtemp: function (prefix, options) {
            return promisify(vol, 'mkdtemp')(prefix, options);
        },
        open: function (path, flags, mode) {
            return promisify(vol, 'open', function (fd) { return new FileHandle(vol, fd); })(path, flags, mode);
        },
        readdir: function (path, options) {
            return promisify(vol, 'readdir')(path, options);
        },
        readFile: function (id, options) {
            return promisify(vol, 'readFile')(id instanceof FileHandle ? id.fd : id, options);
        },
        readlink: function (path, options) {
            return promisify(vol, 'readlink')(path, options);
        },
        realpath: function (path, options) {
            return promisify(vol, 'realpath')(path, options);
        },
        rename: function (oldPath, newPath) {
            return promisify(vol, 'rename')(oldPath, newPath);
        },
        rmdir: function (path) {
            return promisify(vol, 'rmdir')(path);
        },
        rm: function (path, options) {
            return promisify(vol, 'rm')(path, options);
        },
        stat: function (path, options) {
            return promisify(vol, 'stat')(path, options);
        },
        symlink: function (target, path, type) {
            return promisify(vol, 'symlink')(target, path, type);
        },
        truncate: function (path, len) {
            return promisify(vol, 'truncate')(path, len);
        },
        unlink: function (path) {
            return promisify(vol, 'unlink')(path);
        },
        utimes: function (path, atime, mtime) {
            return promisify(vol, 'utimes')(path, atime, mtime);
        },
        writeFile: function (id, data, options) {
            return promisify(vol, 'writeFile')(id instanceof FileHandle ? id.fd : id, data, options);
        },
    };
}
exports["default"] = createPromisesApi;


/***/ }),

/***/ "../../node_modules/memfs/lib/setImmediate.js":
/*!****************************************************!*\
  !*** ../../node_modules/memfs/lib/setImmediate.js ***!
  \****************************************************/
/***/ ((__unused_webpack_module, exports) => {

"use strict";

Object.defineProperty(exports, "__esModule", ({ value: true }));
var _setImmediate;
if (typeof setImmediate === 'function')
    _setImmediate = setImmediate.bind(typeof globalThis !== 'undefined' ? globalThis : global);
else
    _setImmediate = setTimeout.bind(typeof globalThis !== 'undefined' ? globalThis : global);
exports["default"] = _setImmediate;


/***/ }),

/***/ "../../node_modules/memfs/lib/setTimeoutUnref.js":
/*!*******************************************************!*\
  !*** ../../node_modules/memfs/lib/setTimeoutUnref.js ***!
  \*******************************************************/
/***/ ((__unused_webpack_module, exports) => {

"use strict";

Object.defineProperty(exports, "__esModule", ({ value: true }));
/**
 * `setTimeoutUnref` is just like `setTimeout`,
 * only in Node's environment it will "unref" its macro task.
 */
function setTimeoutUnref(callback, time, args) {
    var ref = setTimeout.apply(typeof globalThis !== 'undefined' ? globalThis : global, arguments);
    if (ref && typeof ref === 'object' && typeof ref.unref === 'function')
        ref.unref();
    return ref;
}
exports["default"] = setTimeoutUnref;


/***/ }),

/***/ "../../node_modules/memfs/lib/volume.js":
/*!**********************************************!*\
  !*** ../../node_modules/memfs/lib/volume.js ***!
  \**********************************************/
/***/ (function(__unused_webpack_module, exports, __webpack_require__) {

"use strict";

var __extends = (this && this.__extends) || (function () {
    var extendStatics = function (d, b) {
        extendStatics = Object.setPrototypeOf ||
            ({ __proto__: [] } instanceof Array && function (d, b) { d.__proto__ = b; }) ||
            function (d, b) { for (var p in b) if (Object.prototype.hasOwnProperty.call(b, p)) d[p] = b[p]; };
        return extendStatics(d, b);
    };
    return function (d, b) {
        if (typeof b !== "function" && b !== null)
            throw new TypeError("Class extends value " + String(b) + " is not a constructor or null");
        extendStatics(d, b);
        function __() { this.constructor = d; }
        d.prototype = b === null ? Object.create(b) : (__.prototype = b.prototype, new __());
    };
})();
var __spreadArray = (this && this.__spreadArray) || function (to, from, pack) {
    if (pack || arguments.length === 2) for (var i = 0, l = from.length, ar; i < l; i++) {
        if (ar || !(i in from)) {
            if (!ar) ar = Array.prototype.slice.call(from, 0, i);
            ar[i] = from[i];
        }
    }
    return to.concat(ar || Array.prototype.slice.call(from));
};
Object.defineProperty(exports, "__esModule", ({ value: true }));
exports.FSWatcher = exports.StatWatcher = exports.Volume = exports.toUnixTimestamp = exports.bufferToEncoding = exports.dataToBuffer = exports.dataToStr = exports.pathToSteps = exports.filenameToSteps = exports.pathToFilename = exports.flagsToNumber = exports.FLAGS = void 0;
var pathModule = __webpack_require__(/*! path */ "path");
var node_1 = __webpack_require__(/*! ./node */ "../../node_modules/memfs/lib/node.js");
var Stats_1 = __webpack_require__(/*! ./Stats */ "../../node_modules/memfs/lib/Stats.js");
var Dirent_1 = __webpack_require__(/*! ./Dirent */ "../../node_modules/memfs/lib/Dirent.js");
var buffer_1 = __webpack_require__(/*! ./internal/buffer */ "../../node_modules/memfs/lib/internal/buffer.js");
var setImmediate_1 = __webpack_require__(/*! ./setImmediate */ "../../node_modules/memfs/lib/setImmediate.js");
var process_1 = __webpack_require__(/*! ./process */ "../../node_modules/memfs/lib/process.js");
var setTimeoutUnref_1 = __webpack_require__(/*! ./setTimeoutUnref */ "../../node_modules/memfs/lib/setTimeoutUnref.js");
var stream_1 = __webpack_require__(/*! stream */ "stream");
var constants_1 = __webpack_require__(/*! ./constants */ "../../node_modules/memfs/lib/constants.js");
var events_1 = __webpack_require__(/*! events */ "events");
var encoding_1 = __webpack_require__(/*! ./encoding */ "../../node_modules/memfs/lib/encoding.js");
var errors = __webpack_require__(/*! ./internal/errors */ "../../node_modules/memfs/lib/internal/errors.js");
var util = __webpack_require__(/*! util */ "util");
var promises_1 = __webpack_require__(/*! ./promises */ "../../node_modules/memfs/lib/promises.js");
var resolveCrossPlatform = pathModule.resolve;
var O_RDONLY = constants_1.constants.O_RDONLY, O_WRONLY = constants_1.constants.O_WRONLY, O_RDWR = constants_1.constants.O_RDWR, O_CREAT = constants_1.constants.O_CREAT, O_EXCL = constants_1.constants.O_EXCL, O_TRUNC = constants_1.constants.O_TRUNC, O_APPEND = constants_1.constants.O_APPEND, O_SYNC = constants_1.constants.O_SYNC, O_DIRECTORY = constants_1.constants.O_DIRECTORY, F_OK = constants_1.constants.F_OK, COPYFILE_EXCL = constants_1.constants.COPYFILE_EXCL, COPYFILE_FICLONE_FORCE = constants_1.constants.COPYFILE_FICLONE_FORCE;
var _a = pathModule.posix ? pathModule.posix : pathModule, sep = _a.sep, relative = _a.relative, join = _a.join, dirname = _a.dirname;
var isWin = process_1.default.platform === 'win32';
var kMinPoolSpace = 128;
// const kMaxLength = require('buffer').kMaxLength;
// ---------------------------------------- Error messages
// TODO: Use `internal/errors.js` in the future.
var ERRSTR = {
    PATH_STR: 'path must be a string or Buffer',
    // FD:             'file descriptor must be a unsigned 32-bit integer',
    FD: 'fd must be a file descriptor',
    MODE_INT: 'mode must be an int',
    CB: 'callback must be a function',
    UID: 'uid must be an unsigned int',
    GID: 'gid must be an unsigned int',
    LEN: 'len must be an integer',
    ATIME: 'atime must be an integer',
    MTIME: 'mtime must be an integer',
    PREFIX: 'filename prefix is required',
    BUFFER: 'buffer must be an instance of Buffer or StaticBuffer',
    OFFSET: 'offset must be an integer',
    LENGTH: 'length must be an integer',
    POSITION: 'position must be an integer',
};
var ERRSTR_OPTS = function (tipeof) { return "Expected options to be either an object or a string, but got ".concat(tipeof, " instead"); };
// const ERRSTR_FLAG = flag => `Unknown file open flag: ${flag}`;
var ENOENT = 'ENOENT';
var EBADF = 'EBADF';
var EINVAL = 'EINVAL';
var EPERM = 'EPERM';
var EPROTO = 'EPROTO';
var EEXIST = 'EEXIST';
var ENOTDIR = 'ENOTDIR';
var EMFILE = 'EMFILE';
var EACCES = 'EACCES';
var EISDIR = 'EISDIR';
var ENOTEMPTY = 'ENOTEMPTY';
var ENOSYS = 'ENOSYS';
var ERR_FS_EISDIR = 'ERR_FS_EISDIR';
function formatError(errorCode, func, path, path2) {
    if (func === void 0) { func = ''; }
    if (path === void 0) { path = ''; }
    if (path2 === void 0) { path2 = ''; }
    var pathFormatted = '';
    if (path)
        pathFormatted = " '".concat(path, "'");
    if (path2)
        pathFormatted += " -> '".concat(path2, "'");
    switch (errorCode) {
        case ENOENT:
            return "ENOENT: no such file or directory, ".concat(func).concat(pathFormatted);
        case EBADF:
            return "EBADF: bad file descriptor, ".concat(func).concat(pathFormatted);
        case EINVAL:
            return "EINVAL: invalid argument, ".concat(func).concat(pathFormatted);
        case EPERM:
            return "EPERM: operation not permitted, ".concat(func).concat(pathFormatted);
        case EPROTO:
            return "EPROTO: protocol error, ".concat(func).concat(pathFormatted);
        case EEXIST:
            return "EEXIST: file already exists, ".concat(func).concat(pathFormatted);
        case ENOTDIR:
            return "ENOTDIR: not a directory, ".concat(func).concat(pathFormatted);
        case EISDIR:
            return "EISDIR: illegal operation on a directory, ".concat(func).concat(pathFormatted);
        case EACCES:
            return "EACCES: permission denied, ".concat(func).concat(pathFormatted);
        case ENOTEMPTY:
            return "ENOTEMPTY: directory not empty, ".concat(func).concat(pathFormatted);
        case EMFILE:
            return "EMFILE: too many open files, ".concat(func).concat(pathFormatted);
        case ENOSYS:
            return "ENOSYS: function not implemented, ".concat(func).concat(pathFormatted);
        case ERR_FS_EISDIR:
            return "[ERR_FS_EISDIR]: Path is a directory: ".concat(func, " returned EISDIR (is a directory) ").concat(path);
        default:
            return "".concat(errorCode, ": error occurred, ").concat(func).concat(pathFormatted);
    }
}
function createError(errorCode, func, path, path2, Constructor) {
    if (func === void 0) { func = ''; }
    if (path === void 0) { path = ''; }
    if (path2 === void 0) { path2 = ''; }
    if (Constructor === void 0) { Constructor = Error; }
    var error = new Constructor(formatError(errorCode, func, path, path2));
    error.code = errorCode;
    if (path) {
        error.path = path;
    }
    return error;
}
// ---------------------------------------- Flags
// List of file `flags` as defined by Node.
var FLAGS;
(function (FLAGS) {
    // Open file for reading. An exception occurs if the file does not exist.
    FLAGS[FLAGS["r"] = O_RDONLY] = "r";
    // Open file for reading and writing. An exception occurs if the file does not exist.
    FLAGS[FLAGS["r+"] = O_RDWR] = "r+";
    // Open file for reading in synchronous mode. Instructs the operating system to bypass the local file system cache.
    FLAGS[FLAGS["rs"] = O_RDONLY | O_SYNC] = "rs";
    FLAGS[FLAGS["sr"] = FLAGS.rs] = "sr";
    // Open file for reading and writing, telling the OS to open it synchronously. See notes for 'rs' about using this with caution.
    FLAGS[FLAGS["rs+"] = O_RDWR | O_SYNC] = "rs+";
    FLAGS[FLAGS["sr+"] = FLAGS['rs+']] = "sr+";
    // Open file for writing. The file is created (if it does not exist) or truncated (if it exists).
    FLAGS[FLAGS["w"] = O_WRONLY | O_CREAT | O_TRUNC] = "w";
    // Like 'w' but fails if path exists.
    FLAGS[FLAGS["wx"] = O_WRONLY | O_CREAT | O_TRUNC | O_EXCL] = "wx";
    FLAGS[FLAGS["xw"] = FLAGS.wx] = "xw";
    // Open file for reading and writing. The file is created (if it does not exist) or truncated (if it exists).
    FLAGS[FLAGS["w+"] = O_RDWR | O_CREAT | O_TRUNC] = "w+";
    // Like 'w+' but fails if path exists.
    FLAGS[FLAGS["wx+"] = O_RDWR | O_CREAT | O_TRUNC | O_EXCL] = "wx+";
    FLAGS[FLAGS["xw+"] = FLAGS['wx+']] = "xw+";
    // Open file for appending. The file is created if it does not exist.
    FLAGS[FLAGS["a"] = O_WRONLY | O_APPEND | O_CREAT] = "a";
    // Like 'a' but fails if path exists.
    FLAGS[FLAGS["ax"] = O_WRONLY | O_APPEND | O_CREAT | O_EXCL] = "ax";
    FLAGS[FLAGS["xa"] = FLAGS.ax] = "xa";
    // Open file for reading and appending. The file is created if it does not exist.
    FLAGS[FLAGS["a+"] = O_RDWR | O_APPEND | O_CREAT] = "a+";
    // Like 'a+' but fails if path exists.
    FLAGS[FLAGS["ax+"] = O_RDWR | O_APPEND | O_CREAT | O_EXCL] = "ax+";
    FLAGS[FLAGS["xa+"] = FLAGS['ax+']] = "xa+";
})(FLAGS = exports.FLAGS || (exports.FLAGS = {}));
function flagsToNumber(flags) {
    if (typeof flags === 'number')
        return flags;
    if (typeof flags === 'string') {
        var flagsNum = FLAGS[flags];
        if (typeof flagsNum !== 'undefined')
            return flagsNum;
    }
    // throw new TypeError(formatError(ERRSTR_FLAG(flags)));
    throw new errors.TypeError('ERR_INVALID_OPT_VALUE', 'flags', flags);
}
exports.flagsToNumber = flagsToNumber;
// ---------------------------------------- Options
function getOptions(defaults, options) {
    var opts;
    if (!options)
        return defaults;
    else {
        var tipeof = typeof options;
        switch (tipeof) {
            case 'string':
                opts = Object.assign({}, defaults, { encoding: options });
                break;
            case 'object':
                opts = Object.assign({}, defaults, options);
                break;
            default:
                throw TypeError(ERRSTR_OPTS(tipeof));
        }
    }
    if (opts.encoding !== 'buffer')
        (0, encoding_1.assertEncoding)(opts.encoding);
    return opts;
}
function optsGenerator(defaults) {
    return function (options) { return getOptions(defaults, options); };
}
function validateCallback(callback) {
    if (typeof callback !== 'function')
        throw TypeError(ERRSTR.CB);
    return callback;
}
function optsAndCbGenerator(getOpts) {
    return function (options, callback) {
        return typeof options === 'function' ? [getOpts(), options] : [getOpts(options), validateCallback(callback)];
    };
}
var optsDefaults = {
    encoding: 'utf8',
};
var getDefaultOpts = optsGenerator(optsDefaults);
var getDefaultOptsAndCb = optsAndCbGenerator(getDefaultOpts);
var readFileOptsDefaults = {
    flag: 'r',
};
var getReadFileOptions = optsGenerator(readFileOptsDefaults);
var writeFileDefaults = {
    encoding: 'utf8',
    mode: 438 /* MODE.DEFAULT */,
    flag: FLAGS[FLAGS.w],
};
var getWriteFileOptions = optsGenerator(writeFileDefaults);
var appendFileDefaults = {
    encoding: 'utf8',
    mode: 438 /* MODE.DEFAULT */,
    flag: FLAGS[FLAGS.a],
};
var getAppendFileOpts = optsGenerator(appendFileDefaults);
var getAppendFileOptsAndCb = optsAndCbGenerator(getAppendFileOpts);
var realpathDefaults = optsDefaults;
var getRealpathOptions = optsGenerator(realpathDefaults);
var getRealpathOptsAndCb = optsAndCbGenerator(getRealpathOptions);
var mkdirDefaults = {
    mode: 511 /* MODE.DIR */,
    recursive: false,
};
var getMkdirOptions = function (options) {
    if (typeof options === 'number')
        return Object.assign({}, mkdirDefaults, { mode: options });
    return Object.assign({}, mkdirDefaults, options);
};
var rmdirDefaults = {
    recursive: false,
};
var getRmdirOptions = function (options) {
    return Object.assign({}, rmdirDefaults, options);
};
var getRmOpts = optsGenerator(optsDefaults);
var getRmOptsAndCb = optsAndCbGenerator(getRmOpts);
var readdirDefaults = {
    encoding: 'utf8',
    withFileTypes: false,
};
var getReaddirOptions = optsGenerator(readdirDefaults);
var getReaddirOptsAndCb = optsAndCbGenerator(getReaddirOptions);
var statDefaults = {
    bigint: false,
};
var getStatOptions = function (options) {
    if (options === void 0) { options = {}; }
    return Object.assign({}, statDefaults, options);
};
var getStatOptsAndCb = function (options, callback) {
    return typeof options === 'function' ? [getStatOptions(), options] : [getStatOptions(options), validateCallback(callback)];
};
// ---------------------------------------- Utility functions
function getPathFromURLPosix(url) {
    if (url.hostname !== '') {
        throw new errors.TypeError('ERR_INVALID_FILE_URL_HOST', process_1.default.platform);
    }
    var pathname = url.pathname;
    for (var n = 0; n < pathname.length; n++) {
        if (pathname[n] === '%') {
            var third = pathname.codePointAt(n + 2) | 0x20;
            if (pathname[n + 1] === '2' && third === 102) {
                throw new errors.TypeError('ERR_INVALID_FILE_URL_PATH', 'must not include encoded / characters');
            }
        }
    }
    return decodeURIComponent(pathname);
}
function pathToFilename(path) {
    if (typeof path !== 'string' && !buffer_1.Buffer.isBuffer(path)) {
        try {
            if (!(path instanceof URL))
                throw new TypeError(ERRSTR.PATH_STR);
        }
        catch (err) {
            throw new TypeError(ERRSTR.PATH_STR);
        }
        path = getPathFromURLPosix(path);
    }
    var pathString = String(path);
    nullCheck(pathString);
    // return slash(pathString);
    return pathString;
}
exports.pathToFilename = pathToFilename;
var resolve = function (filename, base) {
    if (base === void 0) { base = process_1.default.cwd(); }
    return resolveCrossPlatform(base, filename);
};
if (isWin) {
    var _resolve_1 = resolve;
    var unixify_1 = (__webpack_require__(/*! fs-monkey/lib/correctPath */ "../../node_modules/fs-monkey/lib/correctPath.js").unixify);
    resolve = function (filename, base) { return unixify_1(_resolve_1(filename, base)); };
}
function filenameToSteps(filename, base) {
    var fullPath = resolve(filename, base);
    var fullPathSansSlash = fullPath.substring(1);
    if (!fullPathSansSlash)
        return [];
    return fullPathSansSlash.split(sep);
}
exports.filenameToSteps = filenameToSteps;
function pathToSteps(path) {
    return filenameToSteps(pathToFilename(path));
}
exports.pathToSteps = pathToSteps;
function dataToStr(data, encoding) {
    if (encoding === void 0) { encoding = encoding_1.ENCODING_UTF8; }
    if (buffer_1.Buffer.isBuffer(data))
        return data.toString(encoding);
    else if (data instanceof Uint8Array)
        return (0, buffer_1.bufferFrom)(data).toString(encoding);
    else
        return String(data);
}
exports.dataToStr = dataToStr;
function dataToBuffer(data, encoding) {
    if (encoding === void 0) { encoding = encoding_1.ENCODING_UTF8; }
    if (buffer_1.Buffer.isBuffer(data))
        return data;
    else if (data instanceof Uint8Array)
        return (0, buffer_1.bufferFrom)(data);
    else
        return (0, buffer_1.bufferFrom)(String(data), encoding);
}
exports.dataToBuffer = dataToBuffer;
function bufferToEncoding(buffer, encoding) {
    if (!encoding || encoding === 'buffer')
        return buffer;
    else
        return buffer.toString(encoding);
}
exports.bufferToEncoding = bufferToEncoding;
function nullCheck(path, callback) {
    if (('' + path).indexOf('\u0000') !== -1) {
        var er = new Error('Path must be a string without null bytes');
        er.code = ENOENT;
        if (typeof callback !== 'function')
            throw er;
        process_1.default.nextTick(callback, er);
        return false;
    }
    return true;
}
function _modeToNumber(mode, def) {
    if (typeof mode === 'number')
        return mode;
    if (typeof mode === 'string')
        return parseInt(mode, 8);
    if (def)
        return modeToNumber(def);
    return undefined;
}
function modeToNumber(mode, def) {
    var result = _modeToNumber(mode, def);
    if (typeof result !== 'number' || isNaN(result))
        throw new TypeError(ERRSTR.MODE_INT);
    return result;
}
function isFd(path) {
    return path >>> 0 === path;
}
function validateFd(fd) {
    if (!isFd(fd))
        throw TypeError(ERRSTR.FD);
}
// converts Date or number to a fractional UNIX timestamp
function toUnixTimestamp(time) {
    // tslint:disable-next-line triple-equals
    if (typeof time === 'string' && +time == time) {
        return +time;
    }
    if (time instanceof Date) {
        return time.getTime() / 1000;
    }
    if (isFinite(time)) {
        if (time < 0) {
            return Date.now() / 1000;
        }
        return time;
    }
    throw new Error('Cannot parse time: ' + time);
}
exports.toUnixTimestamp = toUnixTimestamp;
function validateUid(uid) {
    if (typeof uid !== 'number')
        throw TypeError(ERRSTR.UID);
}
function validateGid(gid) {
    if (typeof gid !== 'number')
        throw TypeError(ERRSTR.GID);
}
function flattenJSON(nestedJSON) {
    var flatJSON = {};
    function flatten(pathPrefix, node) {
        for (var path in node) {
            var contentOrNode = node[path];
            var joinedPath = join(pathPrefix, path);
            if (typeof contentOrNode === 'string') {
                flatJSON[joinedPath] = contentOrNode;
            }
            else if (typeof contentOrNode === 'object' && contentOrNode !== null && Object.keys(contentOrNode).length > 0) {
                // empty directories need an explicit entry and therefore get handled in `else`, non-empty ones are implicitly considered
                flatten(joinedPath, contentOrNode);
            }
            else {
                // without this branch null, empty-object or non-object entries would not be handled in the same way
                // by both fromJSON() and fromNestedJSON()
                flatJSON[joinedPath] = null;
            }
        }
    }
    flatten('', nestedJSON);
    return flatJSON;
}
/**
 * `Volume` represents a file system.
 */
var Volume = /** @class */ (function () {
    function Volume(props) {
        if (props === void 0) { props = {}; }
        // I-node number counter.
        this.ino = 0;
        // A mapping for i-node numbers to i-nodes (`Node`);
        this.inodes = {};
        // List of released i-node numbers, for reuse.
        this.releasedInos = [];
        // A mapping for file descriptors to `File`s.
        this.fds = {};
        // A list of reusable (opened and closed) file descriptors, that should be
        // used first before creating a new file descriptor.
        this.releasedFds = [];
        // Max number of open files.
        this.maxFiles = 10000;
        // Current number of open files.
        this.openFiles = 0;
        this.promisesApi = (0, promises_1.default)(this);
        this.statWatchers = {};
        this.props = Object.assign({ Node: node_1.Node, Link: node_1.Link, File: node_1.File }, props);
        var root = this.createLink();
        root.setNode(this.createNode(true));
        var self = this; // tslint:disable-line no-this-assignment
        this.StatWatcher = /** @class */ (function (_super) {
            __extends(StatWatcher, _super);
            function StatWatcher() {
                return _super.call(this, self) || this;
            }
            return StatWatcher;
        }(StatWatcher));
        var _ReadStream = FsReadStream;
        this.ReadStream = /** @class */ (function (_super) {
            __extends(class_1, _super);
            function class_1() {
                var args = [];
                for (var _i = 0; _i < arguments.length; _i++) {
                    args[_i] = arguments[_i];
                }
                return _super.apply(this, __spreadArray([self], args, false)) || this;
            }
            return class_1;
        }(_ReadStream));
        var _WriteStream = FsWriteStream;
        this.WriteStream = /** @class */ (function (_super) {
            __extends(class_2, _super);
            function class_2() {
                var args = [];
                for (var _i = 0; _i < arguments.length; _i++) {
                    args[_i] = arguments[_i];
                }
                return _super.apply(this, __spreadArray([self], args, false)) || this;
            }
            return class_2;
        }(_WriteStream));
        this.FSWatcher = /** @class */ (function (_super) {
            __extends(FSWatcher, _super);
            function FSWatcher() {
                return _super.call(this, self) || this;
            }
            return FSWatcher;
        }(FSWatcher));
        root.setChild('.', root);
        root.getNode().nlink++;
        root.setChild('..', root);
        root.getNode().nlink++;
        this.root = root;
    }
    Volume.fromJSON = function (json, cwd) {
        var vol = new Volume();
        vol.fromJSON(json, cwd);
        return vol;
    };
    Volume.fromNestedJSON = function (json, cwd) {
        var vol = new Volume();
        vol.fromNestedJSON(json, cwd);
        return vol;
    };
    Object.defineProperty(Volume.prototype, "promises", {
        get: function () {
            if (this.promisesApi === null)
                throw new Error('Promise is not supported in this environment.');
            return this.promisesApi;
        },
        enumerable: false,
        configurable: true
    });
    Volume.prototype.createLink = function (parent, name, isDirectory, perm) {
        if (isDirectory === void 0) { isDirectory = false; }
        if (!parent) {
            return new this.props.Link(this, null, '');
        }
        if (!name) {
            throw new Error('createLink: name cannot be empty');
        }
        return parent.createChild(name, this.createNode(isDirectory, perm));
    };
    Volume.prototype.deleteLink = function (link) {
        var parent = link.parent;
        if (parent) {
            parent.deleteChild(link);
            return true;
        }
        return false;
    };
    Volume.prototype.newInoNumber = function () {
        var releasedFd = this.releasedInos.pop();
        if (releasedFd)
            return releasedFd;
        else {
            this.ino = (this.ino + 1) % 0xffffffff;
            return this.ino;
        }
    };
    Volume.prototype.newFdNumber = function () {
        var releasedFd = this.releasedFds.pop();
        return typeof releasedFd === 'number' ? releasedFd : Volume.fd--;
    };
    Volume.prototype.createNode = function (isDirectory, perm) {
        if (isDirectory === void 0) { isDirectory = false; }
        var node = new this.props.Node(this.newInoNumber(), perm);
        if (isDirectory)
            node.setIsDirectory();
        this.inodes[node.ino] = node;
        return node;
    };
    Volume.prototype.getNode = function (ino) {
        return this.inodes[ino];
    };
    Volume.prototype.deleteNode = function (node) {
        node.del();
        delete this.inodes[node.ino];
        this.releasedInos.push(node.ino);
    };
    // Generates 6 character long random string, used by `mkdtemp`.
    Volume.prototype.genRndStr = function () {
        var str = (Math.random() + 1).toString(36).substring(2, 8);
        if (str.length === 6)
            return str;
        else
            return this.genRndStr();
    };
    // Returns a `Link` (hard link) referenced by path "split" into steps.
    Volume.prototype.getLink = function (steps) {
        return this.root.walk(steps);
    };
    // Just link `getLink`, but throws a correct user error, if link to found.
    Volume.prototype.getLinkOrThrow = function (filename, funcName) {
        var steps = filenameToSteps(filename);
        var link = this.getLink(steps);
        if (!link)
            throw createError(ENOENT, funcName, filename);
        return link;
    };
    // Just like `getLink`, but also dereference/resolves symbolic links.
    Volume.prototype.getResolvedLink = function (filenameOrSteps) {
        var steps = typeof filenameOrSteps === 'string' ? filenameToSteps(filenameOrSteps) : filenameOrSteps;
        var link = this.root;
        var i = 0;
        while (i < steps.length) {
            var step = steps[i];
            link = link.getChild(step);
            if (!link)
                return null;
            var node = link.getNode();
            if (node.isSymlink()) {
                steps = node.symlink.concat(steps.slice(i + 1));
                link = this.root;
                i = 0;
                continue;
            }
            i++;
        }
        return link;
    };
    // Just like `getLinkOrThrow`, but also dereference/resolves symbolic links.
    Volume.prototype.getResolvedLinkOrThrow = function (filename, funcName) {
        var link = this.getResolvedLink(filename);
        if (!link)
            throw createError(ENOENT, funcName, filename);
        return link;
    };
    Volume.prototype.resolveSymlinks = function (link) {
        // let node: Node = link.getNode();
        // while(link && node.isSymlink()) {
        //     link = this.getLink(node.symlink);
        //     if(!link) return null;
        //     node = link.getNode();
        // }
        // return link;
        return this.getResolvedLink(link.steps.slice(1));
    };
    // Just like `getLinkOrThrow`, but also verifies that the link is a directory.
    Volume.prototype.getLinkAsDirOrThrow = function (filename, funcName) {
        var link = this.getLinkOrThrow(filename, funcName);
        if (!link.getNode().isDirectory())
            throw createError(ENOTDIR, funcName, filename);
        return link;
    };
    // Get the immediate parent directory of the link.
    Volume.prototype.getLinkParent = function (steps) {
        return this.root.walk(steps, steps.length - 1);
    };
    Volume.prototype.getLinkParentAsDirOrThrow = function (filenameOrSteps, funcName) {
        var steps = filenameOrSteps instanceof Array ? filenameOrSteps : filenameToSteps(filenameOrSteps);
        var link = this.getLinkParent(steps);
        if (!link)
            throw createError(ENOENT, funcName, sep + steps.join(sep));
        if (!link.getNode().isDirectory())
            throw createError(ENOTDIR, funcName, sep + steps.join(sep));
        return link;
    };
    Volume.prototype.getFileByFd = function (fd) {
        return this.fds[String(fd)];
    };
    Volume.prototype.getFileByFdOrThrow = function (fd, funcName) {
        if (!isFd(fd))
            throw TypeError(ERRSTR.FD);
        var file = this.getFileByFd(fd);
        if (!file)
            throw createError(EBADF, funcName);
        return file;
    };
    /**
     * @todo This is not used anymore. Remove.
     */
    /*
    private getNodeByIdOrCreate(id: TFileId, flags: number, perm: number): Node {
      if (typeof id === 'number') {
        const file = this.getFileByFd(id);
        if (!file) throw Error('File nto found');
        return file.node;
      } else {
        const steps = pathToSteps(id as PathLike);
        let link = this.getLink(steps);
        if (link) return link.getNode();
  
        // Try creating a node if not found.
        if (flags & O_CREAT) {
          const dirLink = this.getLinkParent(steps);
          if (dirLink) {
            const name = steps[steps.length - 1];
            link = this.createLink(dirLink, name, false, perm);
            return link.getNode();
          }
        }
  
        throw createError(ENOENT, 'getNodeByIdOrCreate', pathToFilename(id));
      }
    }
    */
    Volume.prototype.wrapAsync = function (method, args, callback) {
        var _this = this;
        validateCallback(callback);
        (0, setImmediate_1.default)(function () {
            var result;
            try {
                result = method.apply(_this, args);
            }
            catch (err) {
                callback(err);
                return;
            }
            callback(null, result);
        });
    };
    Volume.prototype._toJSON = function (link, json, path) {
        var _a;
        if (link === void 0) { link = this.root; }
        if (json === void 0) { json = {}; }
        var isEmpty = true;
        var children = link.children;
        if (link.getNode().isFile()) {
            children = (_a = {}, _a[link.getName()] = link.parent.getChild(link.getName()), _a);
            link = link.parent;
        }
        for (var name_1 in children) {
            if (name_1 === '.' || name_1 === '..') {
                continue;
            }
            isEmpty = false;
            var child = link.getChild(name_1);
            if (!child) {
                throw new Error('_toJSON: unexpected undefined');
            }
            var node = child.getNode();
            if (node.isFile()) {
                var filename = child.getPath();
                if (path)
                    filename = relative(path, filename);
                json[filename] = node.getString();
            }
            else if (node.isDirectory()) {
                this._toJSON(child, json, path);
            }
        }
        var dirPath = link.getPath();
        if (path)
            dirPath = relative(path, dirPath);
        if (dirPath && isEmpty) {
            json[dirPath] = null;
        }
        return json;
    };
    Volume.prototype.toJSON = function (paths, json, isRelative) {
        if (json === void 0) { json = {}; }
        if (isRelative === void 0) { isRelative = false; }
        var links = [];
        if (paths) {
            if (!(paths instanceof Array))
                paths = [paths];
            for (var _i = 0, paths_1 = paths; _i < paths_1.length; _i++) {
                var path = paths_1[_i];
                var filename = pathToFilename(path);
                var link = this.getResolvedLink(filename);
                if (!link)
                    continue;
                links.push(link);
            }
        }
        else {
            links.push(this.root);
        }
        if (!links.length)
            return json;
        for (var _a = 0, links_1 = links; _a < links_1.length; _a++) {
            var link = links_1[_a];
            this._toJSON(link, json, isRelative ? link.getPath() : '');
        }
        return json;
    };
    Volume.prototype.fromJSON = function (json, cwd) {
        if (cwd === void 0) { cwd = process_1.default.cwd(); }
        for (var filename in json) {
            var data = json[filename];
            filename = resolve(filename, cwd);
            if (typeof data === 'string') {
                var dir = dirname(filename);
                this.mkdirpBase(dir, 511 /* MODE.DIR */);
                this.writeFileSync(filename, data);
            }
            else {
                this.mkdirpBase(filename, 511 /* MODE.DIR */);
            }
        }
    };
    Volume.prototype.fromNestedJSON = function (json, cwd) {
        this.fromJSON(flattenJSON(json), cwd);
    };
    Volume.prototype.reset = function () {
        this.ino = 0;
        this.inodes = {};
        this.releasedInos = [];
        this.fds = {};
        this.releasedFds = [];
        this.openFiles = 0;
        this.root = this.createLink();
        this.root.setNode(this.createNode(true));
    };
    // Legacy interface
    Volume.prototype.mountSync = function (mountpoint, json) {
        this.fromJSON(json, mountpoint);
    };
    Volume.prototype.openLink = function (link, flagsNum, resolveSymlinks) {
        if (resolveSymlinks === void 0) { resolveSymlinks = true; }
        if (this.openFiles >= this.maxFiles) {
            // Too many open files.
            throw createError(EMFILE, 'open', link.getPath());
        }
        // Resolve symlinks.
        var realLink = link;
        if (resolveSymlinks)
            realLink = this.resolveSymlinks(link);
        if (!realLink)
            throw createError(ENOENT, 'open', link.getPath());
        var node = realLink.getNode();
        // Check whether node is a directory
        if (node.isDirectory()) {
            if ((flagsNum & (O_RDONLY | O_RDWR | O_WRONLY)) !== O_RDONLY)
                throw createError(EISDIR, 'open', link.getPath());
        }
        else {
            if (flagsNum & O_DIRECTORY)
                throw createError(ENOTDIR, 'open', link.getPath());
        }
        // Check node permissions
        if (!(flagsNum & O_WRONLY)) {
            if (!node.canRead()) {
                throw createError(EACCES, 'open', link.getPath());
            }
        }
        if (flagsNum & O_RDWR) {
        }
        var file = new this.props.File(link, node, flagsNum, this.newFdNumber());
        this.fds[file.fd] = file;
        this.openFiles++;
        if (flagsNum & O_TRUNC)
            file.truncate();
        return file;
    };
    Volume.prototype.openFile = function (filename, flagsNum, modeNum, resolveSymlinks) {
        if (resolveSymlinks === void 0) { resolveSymlinks = true; }
        var steps = filenameToSteps(filename);
        var link = resolveSymlinks ? this.getResolvedLink(steps) : this.getLink(steps);
        if (link && flagsNum & O_EXCL)
            throw createError(EEXIST, 'open', filename);
        // Try creating a new file, if it does not exist.
        if (!link && flagsNum & O_CREAT) {
            // const dirLink: Link = this.getLinkParent(steps);
            var dirLink = this.getResolvedLink(steps.slice(0, steps.length - 1));
            // if(!dirLink) throw createError(ENOENT, 'open', filename);
            if (!dirLink)
                throw createError(ENOENT, 'open', sep + steps.join(sep));
            if (flagsNum & O_CREAT && typeof modeNum === 'number') {
                link = this.createLink(dirLink, steps[steps.length - 1], false, modeNum);
            }
        }
        if (link)
            return this.openLink(link, flagsNum, resolveSymlinks);
        throw createError(ENOENT, 'open', filename);
    };
    Volume.prototype.openBase = function (filename, flagsNum, modeNum, resolveSymlinks) {
        if (resolveSymlinks === void 0) { resolveSymlinks = true; }
        var file = this.openFile(filename, flagsNum, modeNum, resolveSymlinks);
        if (!file)
            throw createError(ENOENT, 'open', filename);
        return file.fd;
    };
    Volume.prototype.openSync = function (path, flags, mode) {
        if (mode === void 0) { mode = 438 /* MODE.DEFAULT */; }
        // Validate (1) mode; (2) path; (3) flags - in that order.
        var modeNum = modeToNumber(mode);
        var fileName = pathToFilename(path);
        var flagsNum = flagsToNumber(flags);
        return this.openBase(fileName, flagsNum, modeNum);
    };
    Volume.prototype.open = function (path, flags, a, b) {
        var mode = a;
        var callback = b;
        if (typeof a === 'function') {
            mode = 438 /* MODE.DEFAULT */;
            callback = a;
        }
        mode = mode || 438 /* MODE.DEFAULT */;
        var modeNum = modeToNumber(mode);
        var fileName = pathToFilename(path);
        var flagsNum = flagsToNumber(flags);
        this.wrapAsync(this.openBase, [fileName, flagsNum, modeNum], callback);
    };
    Volume.prototype.closeFile = function (file) {
        if (!this.fds[file.fd])
            return;
        this.openFiles--;
        delete this.fds[file.fd];
        this.releasedFds.push(file.fd);
    };
    Volume.prototype.closeSync = function (fd) {
        validateFd(fd);
        var file = this.getFileByFdOrThrow(fd, 'close');
        this.closeFile(file);
    };
    Volume.prototype.close = function (fd, callback) {
        validateFd(fd);
        this.wrapAsync(this.closeSync, [fd], callback);
    };
    Volume.prototype.openFileOrGetById = function (id, flagsNum, modeNum) {
        if (typeof id === 'number') {
            var file = this.fds[id];
            if (!file)
                throw createError(ENOENT);
            return file;
        }
        else {
            return this.openFile(pathToFilename(id), flagsNum, modeNum);
        }
    };
    Volume.prototype.readBase = function (fd, buffer, offset, length, position) {
        var file = this.getFileByFdOrThrow(fd);
        return file.read(buffer, Number(offset), Number(length), position);
    };
    Volume.prototype.readSync = function (fd, buffer, offset, length, position) {
        validateFd(fd);
        return this.readBase(fd, buffer, offset, length, position);
    };
    Volume.prototype.read = function (fd, buffer, offset, length, position, callback) {
        var _this = this;
        validateCallback(callback);
        // This `if` branch is from Node.js
        if (length === 0) {
            return process_1.default.nextTick(function () {
                if (callback)
                    callback(null, 0, buffer);
            });
        }
        (0, setImmediate_1.default)(function () {
            try {
                var bytes = _this.readBase(fd, buffer, offset, length, position);
                callback(null, bytes, buffer);
            }
            catch (err) {
                callback(err);
            }
        });
    };
    Volume.prototype.readFileBase = function (id, flagsNum, encoding) {
        var result;
        var isUserFd = typeof id === 'number';
        var userOwnsFd = isUserFd && isFd(id);
        var fd;
        if (userOwnsFd)
            fd = id;
        else {
            var filename = pathToFilename(id);
            var steps = filenameToSteps(filename);
            var link = this.getResolvedLink(steps);
            if (link) {
                var node = link.getNode();
                if (node.isDirectory())
                    throw createError(EISDIR, 'open', link.getPath());
            }
            fd = this.openSync(id, flagsNum);
        }
        try {
            result = bufferToEncoding(this.getFileByFdOrThrow(fd).getBuffer(), encoding);
        }
        finally {
            if (!userOwnsFd) {
                this.closeSync(fd);
            }
        }
        return result;
    };
    Volume.prototype.readFileSync = function (file, options) {
        var opts = getReadFileOptions(options);
        var flagsNum = flagsToNumber(opts.flag);
        return this.readFileBase(file, flagsNum, opts.encoding);
    };
    Volume.prototype.readFile = function (id, a, b) {
        var _a = optsAndCbGenerator(getReadFileOptions)(a, b), opts = _a[0], callback = _a[1];
        var flagsNum = flagsToNumber(opts.flag);
        this.wrapAsync(this.readFileBase, [id, flagsNum, opts.encoding], callback);
    };
    Volume.prototype.writeBase = function (fd, buf, offset, length, position) {
        var file = this.getFileByFdOrThrow(fd, 'write');
        return file.write(buf, offset, length, position);
    };
    Volume.prototype.writeSync = function (fd, a, b, c, d) {
        validateFd(fd);
        var encoding;
        var offset;
        var length;
        var position;
        var isBuffer = typeof a !== 'string';
        if (isBuffer) {
            offset = (b || 0) | 0;
            length = c;
            position = d;
        }
        else {
            position = b;
            encoding = c;
        }
        var buf = dataToBuffer(a, encoding);
        if (isBuffer) {
            if (typeof length === 'undefined') {
                length = buf.length;
            }
        }
        else {
            offset = 0;
            length = buf.length;
        }
        return this.writeBase(fd, buf, offset, length, position);
    };
    Volume.prototype.write = function (fd, a, b, c, d, e) {
        var _this = this;
        validateFd(fd);
        var offset;
        var length;
        var position;
        var encoding;
        var callback;
        var tipa = typeof a;
        var tipb = typeof b;
        var tipc = typeof c;
        var tipd = typeof d;
        if (tipa !== 'string') {
            if (tipb === 'function') {
                callback = b;
            }
            else if (tipc === 'function') {
                offset = b | 0;
                callback = c;
            }
            else if (tipd === 'function') {
                offset = b | 0;
                length = c;
                callback = d;
            }
            else {
                offset = b | 0;
                length = c;
                position = d;
                callback = e;
            }
        }
        else {
            if (tipb === 'function') {
                callback = b;
            }
            else if (tipc === 'function') {
                position = b;
                callback = c;
            }
            else if (tipd === 'function') {
                position = b;
                encoding = c;
                callback = d;
            }
        }
        var buf = dataToBuffer(a, encoding);
        if (tipa !== 'string') {
            if (typeof length === 'undefined')
                length = buf.length;
        }
        else {
            offset = 0;
            length = buf.length;
        }
        var cb = validateCallback(callback);
        (0, setImmediate_1.default)(function () {
            try {
                var bytes = _this.writeBase(fd, buf, offset, length, position);
                if (tipa !== 'string') {
                    cb(null, bytes, buf);
                }
                else {
                    cb(null, bytes, a);
                }
            }
            catch (err) {
                cb(err);
            }
        });
    };
    Volume.prototype.writeFileBase = function (id, buf, flagsNum, modeNum) {
        // console.log('writeFileBase', id, buf, flagsNum, modeNum);
        // const node = this.getNodeByIdOrCreate(id, flagsNum, modeNum);
        // node.setBuffer(buf);
        var isUserFd = typeof id === 'number';
        var fd;
        if (isUserFd)
            fd = id;
        else {
            fd = this.openBase(pathToFilename(id), flagsNum, modeNum);
            // fd = this.openSync(id as PathLike, flagsNum, modeNum);
        }
        var offset = 0;
        var length = buf.length;
        var position = flagsNum & O_APPEND ? undefined : 0;
        try {
            while (length > 0) {
                var written = this.writeSync(fd, buf, offset, length, position);
                offset += written;
                length -= written;
                if (position !== undefined)
                    position += written;
            }
        }
        finally {
            if (!isUserFd)
                this.closeSync(fd);
        }
    };
    Volume.prototype.writeFileSync = function (id, data, options) {
        var opts = getWriteFileOptions(options);
        var flagsNum = flagsToNumber(opts.flag);
        var modeNum = modeToNumber(opts.mode);
        var buf = dataToBuffer(data, opts.encoding);
        this.writeFileBase(id, buf, flagsNum, modeNum);
    };
    Volume.prototype.writeFile = function (id, data, a, b) {
        var options = a;
        var callback = b;
        if (typeof a === 'function') {
            options = writeFileDefaults;
            callback = a;
        }
        var cb = validateCallback(callback);
        var opts = getWriteFileOptions(options);
        var flagsNum = flagsToNumber(opts.flag);
        var modeNum = modeToNumber(opts.mode);
        var buf = dataToBuffer(data, opts.encoding);
        this.wrapAsync(this.writeFileBase, [id, buf, flagsNum, modeNum], cb);
    };
    Volume.prototype.linkBase = function (filename1, filename2) {
        var steps1 = filenameToSteps(filename1);
        var link1 = this.getLink(steps1);
        if (!link1)
            throw createError(ENOENT, 'link', filename1, filename2);
        var steps2 = filenameToSteps(filename2);
        // Check new link directory exists.
        var dir2 = this.getLinkParent(steps2);
        if (!dir2)
            throw createError(ENOENT, 'link', filename1, filename2);
        var name = steps2[steps2.length - 1];
        // Check if new file already exists.
        if (dir2.getChild(name))
            throw createError(EEXIST, 'link', filename1, filename2);
        var node = link1.getNode();
        node.nlink++;
        dir2.createChild(name, node);
    };
    Volume.prototype.copyFileBase = function (src, dest, flags) {
        var buf = this.readFileSync(src);
        if (flags & COPYFILE_EXCL) {
            if (this.existsSync(dest)) {
                throw createError(EEXIST, 'copyFile', src, dest);
            }
        }
        if (flags & COPYFILE_FICLONE_FORCE) {
            throw createError(ENOSYS, 'copyFile', src, dest);
        }
        this.writeFileBase(dest, buf, FLAGS.w, 438 /* MODE.DEFAULT */);
    };
    Volume.prototype.copyFileSync = function (src, dest, flags) {
        var srcFilename = pathToFilename(src);
        var destFilename = pathToFilename(dest);
        return this.copyFileBase(srcFilename, destFilename, (flags || 0) | 0);
    };
    Volume.prototype.copyFile = function (src, dest, a, b) {
        var srcFilename = pathToFilename(src);
        var destFilename = pathToFilename(dest);
        var flags;
        var callback;
        if (typeof a === 'function') {
            flags = 0;
            callback = a;
        }
        else {
            flags = a;
            callback = b;
        }
        validateCallback(callback);
        this.wrapAsync(this.copyFileBase, [srcFilename, destFilename, flags], callback);
    };
    Volume.prototype.linkSync = function (existingPath, newPath) {
        var existingPathFilename = pathToFilename(existingPath);
        var newPathFilename = pathToFilename(newPath);
        this.linkBase(existingPathFilename, newPathFilename);
    };
    Volume.prototype.link = function (existingPath, newPath, callback) {
        var existingPathFilename = pathToFilename(existingPath);
        var newPathFilename = pathToFilename(newPath);
        this.wrapAsync(this.linkBase, [existingPathFilename, newPathFilename], callback);
    };
    Volume.prototype.unlinkBase = function (filename) {
        var steps = filenameToSteps(filename);
        var link = this.getLink(steps);
        if (!link)
            throw createError(ENOENT, 'unlink', filename);
        // TODO: Check if it is file, dir, other...
        if (link.length)
            throw Error('Dir not empty...');
        this.deleteLink(link);
        var node = link.getNode();
        node.nlink--;
        // When all hard links to i-node are deleted, remove the i-node, too.
        if (node.nlink <= 0) {
            this.deleteNode(node);
        }
    };
    Volume.prototype.unlinkSync = function (path) {
        var filename = pathToFilename(path);
        this.unlinkBase(filename);
    };
    Volume.prototype.unlink = function (path, callback) {
        var filename = pathToFilename(path);
        this.wrapAsync(this.unlinkBase, [filename], callback);
    };
    Volume.prototype.symlinkBase = function (targetFilename, pathFilename) {
        var pathSteps = filenameToSteps(pathFilename);
        // Check if directory exists, where we about to create a symlink.
        var dirLink = this.getLinkParent(pathSteps);
        if (!dirLink)
            throw createError(ENOENT, 'symlink', targetFilename, pathFilename);
        var name = pathSteps[pathSteps.length - 1];
        // Check if new file already exists.
        if (dirLink.getChild(name))
            throw createError(EEXIST, 'symlink', targetFilename, pathFilename);
        // Create symlink.
        var symlink = dirLink.createChild(name);
        symlink.getNode().makeSymlink(filenameToSteps(targetFilename));
        return symlink;
    };
    // `type` argument works only on Windows.
    Volume.prototype.symlinkSync = function (target, path, type) {
        var targetFilename = pathToFilename(target);
        var pathFilename = pathToFilename(path);
        this.symlinkBase(targetFilename, pathFilename);
    };
    Volume.prototype.symlink = function (target, path, a, b) {
        var callback = validateCallback(typeof a === 'function' ? a : b);
        var targetFilename = pathToFilename(target);
        var pathFilename = pathToFilename(path);
        this.wrapAsync(this.symlinkBase, [targetFilename, pathFilename], callback);
    };
    Volume.prototype.realpathBase = function (filename, encoding) {
        var steps = filenameToSteps(filename);
        var realLink = this.getResolvedLink(steps);
        if (!realLink)
            throw createError(ENOENT, 'realpath', filename);
        return (0, encoding_1.strToEncoding)(realLink.getPath() || '/', encoding);
    };
    Volume.prototype.realpathSync = function (path, options) {
        return this.realpathBase(pathToFilename(path), getRealpathOptions(options).encoding);
    };
    Volume.prototype.realpath = function (path, a, b) {
        var _a = getRealpathOptsAndCb(a, b), opts = _a[0], callback = _a[1];
        var pathFilename = pathToFilename(path);
        this.wrapAsync(this.realpathBase, [pathFilename, opts.encoding], callback);
    };
    Volume.prototype.lstatBase = function (filename, bigint, throwIfNoEntry) {
        if (bigint === void 0) { bigint = false; }
        if (throwIfNoEntry === void 0) { throwIfNoEntry = false; }
        var link = this.getLink(filenameToSteps(filename));
        if (link) {
            return Stats_1.default.build(link.getNode(), bigint);
        }
        else if (!throwIfNoEntry) {
            return undefined;
        }
        else {
            throw createError(ENOENT, 'lstat', filename);
        }
    };
    Volume.prototype.lstatSync = function (path, options) {
        var _a = getStatOptions(options), _b = _a.throwIfNoEntry, throwIfNoEntry = _b === void 0 ? true : _b, _c = _a.bigint, bigint = _c === void 0 ? false : _c;
        return this.lstatBase(pathToFilename(path), bigint, throwIfNoEntry);
    };
    Volume.prototype.lstat = function (path, a, b) {
        var _a = getStatOptsAndCb(a, b), _b = _a[0], _c = _b.throwIfNoEntry, throwIfNoEntry = _c === void 0 ? true : _c, _d = _b.bigint, bigint = _d === void 0 ? false : _d, callback = _a[1];
        this.wrapAsync(this.lstatBase, [pathToFilename(path), bigint, throwIfNoEntry], callback);
    };
    Volume.prototype.statBase = function (filename, bigint, throwIfNoEntry) {
        if (bigint === void 0) { bigint = false; }
        if (throwIfNoEntry === void 0) { throwIfNoEntry = true; }
        var link = this.getResolvedLink(filenameToSteps(filename));
        if (link) {
            return Stats_1.default.build(link.getNode(), bigint);
        }
        else if (!throwIfNoEntry) {
            return undefined;
        }
        else {
            throw createError(ENOENT, 'stat', filename);
        }
    };
    Volume.prototype.statSync = function (path, options) {
        var _a = getStatOptions(options), _b = _a.bigint, bigint = _b === void 0 ? true : _b, _c = _a.throwIfNoEntry, throwIfNoEntry = _c === void 0 ? true : _c;
        return this.statBase(pathToFilename(path), bigint, throwIfNoEntry);
    };
    Volume.prototype.stat = function (path, a, b) {
        var _a = getStatOptsAndCb(a, b), _b = _a[0], _c = _b.bigint, bigint = _c === void 0 ? false : _c, _d = _b.throwIfNoEntry, throwIfNoEntry = _d === void 0 ? true : _d, callback = _a[1];
        this.wrapAsync(this.statBase, [pathToFilename(path), bigint, throwIfNoEntry], callback);
    };
    Volume.prototype.fstatBase = function (fd, bigint) {
        if (bigint === void 0) { bigint = false; }
        var file = this.getFileByFd(fd);
        if (!file)
            throw createError(EBADF, 'fstat');
        return Stats_1.default.build(file.node, bigint);
    };
    Volume.prototype.fstatSync = function (fd, options) {
        return this.fstatBase(fd, getStatOptions(options).bigint);
    };
    Volume.prototype.fstat = function (fd, a, b) {
        var _a = getStatOptsAndCb(a, b), opts = _a[0], callback = _a[1];
        this.wrapAsync(this.fstatBase, [fd, opts.bigint], callback);
    };
    Volume.prototype.renameBase = function (oldPathFilename, newPathFilename) {
        var link = this.getLink(filenameToSteps(oldPathFilename));
        if (!link)
            throw createError(ENOENT, 'rename', oldPathFilename, newPathFilename);
        // TODO: Check if it is directory, if non-empty, we cannot move it, right?
        var newPathSteps = filenameToSteps(newPathFilename);
        // Check directory exists for the new location.
        var newPathDirLink = this.getLinkParent(newPathSteps);
        if (!newPathDirLink)
            throw createError(ENOENT, 'rename', oldPathFilename, newPathFilename);
        // TODO: Also treat cases with directories and symbolic links.
        // TODO: See: http://man7.org/linux/man-pages/man2/rename.2.html
        // Remove hard link from old folder.
        var oldLinkParent = link.parent;
        if (oldLinkParent) {
            oldLinkParent.deleteChild(link);
        }
        // Rename should overwrite the new path, if that exists.
        var name = newPathSteps[newPathSteps.length - 1];
        link.name = name;
        link.steps = __spreadArray(__spreadArray([], newPathDirLink.steps, true), [name], false);
        newPathDirLink.setChild(link.getName(), link);
    };
    Volume.prototype.renameSync = function (oldPath, newPath) {
        var oldPathFilename = pathToFilename(oldPath);
        var newPathFilename = pathToFilename(newPath);
        this.renameBase(oldPathFilename, newPathFilename);
    };
    Volume.prototype.rename = function (oldPath, newPath, callback) {
        var oldPathFilename = pathToFilename(oldPath);
        var newPathFilename = pathToFilename(newPath);
        this.wrapAsync(this.renameBase, [oldPathFilename, newPathFilename], callback);
    };
    Volume.prototype.existsBase = function (filename) {
        return !!this.statBase(filename);
    };
    Volume.prototype.existsSync = function (path) {
        try {
            return this.existsBase(pathToFilename(path));
        }
        catch (err) {
            return false;
        }
    };
    Volume.prototype.exists = function (path, callback) {
        var _this = this;
        var filename = pathToFilename(path);
        if (typeof callback !== 'function')
            throw Error(ERRSTR.CB);
        (0, setImmediate_1.default)(function () {
            try {
                callback(_this.existsBase(filename));
            }
            catch (err) {
                callback(false);
            }
        });
    };
    Volume.prototype.accessBase = function (filename, mode) {
        var link = this.getLinkOrThrow(filename, 'access');
        // TODO: Verify permissions
    };
    Volume.prototype.accessSync = function (path, mode) {
        if (mode === void 0) { mode = F_OK; }
        var filename = pathToFilename(path);
        mode = mode | 0;
        this.accessBase(filename, mode);
    };
    Volume.prototype.access = function (path, a, b) {
        var mode = F_OK;
        var callback;
        if (typeof a !== 'function') {
            mode = a | 0; // cast to number
            callback = validateCallback(b);
        }
        else {
            callback = a;
        }
        var filename = pathToFilename(path);
        this.wrapAsync(this.accessBase, [filename, mode], callback);
    };
    Volume.prototype.appendFileSync = function (id, data, options) {
        if (options === void 0) { options = appendFileDefaults; }
        var opts = getAppendFileOpts(options);
        // force append behavior when using a supplied file descriptor
        if (!opts.flag || isFd(id))
            opts.flag = 'a';
        this.writeFileSync(id, data, opts);
    };
    Volume.prototype.appendFile = function (id, data, a, b) {
        var _a = getAppendFileOptsAndCb(a, b), opts = _a[0], callback = _a[1];
        // force append behavior when using a supplied file descriptor
        if (!opts.flag || isFd(id))
            opts.flag = 'a';
        this.writeFile(id, data, opts, callback);
    };
    Volume.prototype.readdirBase = function (filename, options) {
        var steps = filenameToSteps(filename);
        var link = this.getResolvedLink(steps);
        if (!link)
            throw createError(ENOENT, 'readdir', filename);
        var node = link.getNode();
        if (!node.isDirectory())
            throw createError(ENOTDIR, 'scandir', filename);
        if (options.withFileTypes) {
            var list_1 = [];
            for (var name_2 in link.children) {
                var child = link.getChild(name_2);
                if (!child || name_2 === '.' || name_2 === '..') {
                    continue;
                }
                list_1.push(Dirent_1.default.build(child, options.encoding));
            }
            if (!isWin && options.encoding !== 'buffer')
                list_1.sort(function (a, b) {
                    if (a.name < b.name)
                        return -1;
                    if (a.name > b.name)
                        return 1;
                    return 0;
                });
            return list_1;
        }
        var list = [];
        for (var name_3 in link.children) {
            if (name_3 === '.' || name_3 === '..') {
                continue;
            }
            list.push((0, encoding_1.strToEncoding)(name_3, options.encoding));
        }
        if (!isWin && options.encoding !== 'buffer')
            list.sort();
        return list;
    };
    Volume.prototype.readdirSync = function (path, options) {
        var opts = getReaddirOptions(options);
        var filename = pathToFilename(path);
        return this.readdirBase(filename, opts);
    };
    Volume.prototype.readdir = function (path, a, b) {
        var _a = getReaddirOptsAndCb(a, b), options = _a[0], callback = _a[1];
        var filename = pathToFilename(path);
        this.wrapAsync(this.readdirBase, [filename, options], callback);
    };
    Volume.prototype.readlinkBase = function (filename, encoding) {
        var link = this.getLinkOrThrow(filename, 'readlink');
        var node = link.getNode();
        if (!node.isSymlink())
            throw createError(EINVAL, 'readlink', filename);
        var str = sep + node.symlink.join(sep);
        return (0, encoding_1.strToEncoding)(str, encoding);
    };
    Volume.prototype.readlinkSync = function (path, options) {
        var opts = getDefaultOpts(options);
        var filename = pathToFilename(path);
        return this.readlinkBase(filename, opts.encoding);
    };
    Volume.prototype.readlink = function (path, a, b) {
        var _a = getDefaultOptsAndCb(a, b), opts = _a[0], callback = _a[1];
        var filename = pathToFilename(path);
        this.wrapAsync(this.readlinkBase, [filename, opts.encoding], callback);
    };
    Volume.prototype.fsyncBase = function (fd) {
        this.getFileByFdOrThrow(fd, 'fsync');
    };
    Volume.prototype.fsyncSync = function (fd) {
        this.fsyncBase(fd);
    };
    Volume.prototype.fsync = function (fd, callback) {
        this.wrapAsync(this.fsyncBase, [fd], callback);
    };
    Volume.prototype.fdatasyncBase = function (fd) {
        this.getFileByFdOrThrow(fd, 'fdatasync');
    };
    Volume.prototype.fdatasyncSync = function (fd) {
        this.fdatasyncBase(fd);
    };
    Volume.prototype.fdatasync = function (fd, callback) {
        this.wrapAsync(this.fdatasyncBase, [fd], callback);
    };
    Volume.prototype.ftruncateBase = function (fd, len) {
        var file = this.getFileByFdOrThrow(fd, 'ftruncate');
        file.truncate(len);
    };
    Volume.prototype.ftruncateSync = function (fd, len) {
        this.ftruncateBase(fd, len);
    };
    Volume.prototype.ftruncate = function (fd, a, b) {
        var len = typeof a === 'number' ? a : 0;
        var callback = validateCallback(typeof a === 'number' ? b : a);
        this.wrapAsync(this.ftruncateBase, [fd, len], callback);
    };
    Volume.prototype.truncateBase = function (path, len) {
        var fd = this.openSync(path, 'r+');
        try {
            this.ftruncateSync(fd, len);
        }
        finally {
            this.closeSync(fd);
        }
    };
    Volume.prototype.truncateSync = function (id, len) {
        if (isFd(id))
            return this.ftruncateSync(id, len);
        this.truncateBase(id, len);
    };
    Volume.prototype.truncate = function (id, a, b) {
        var len = typeof a === 'number' ? a : 0;
        var callback = validateCallback(typeof a === 'number' ? b : a);
        if (isFd(id))
            return this.ftruncate(id, len, callback);
        this.wrapAsync(this.truncateBase, [id, len], callback);
    };
    Volume.prototype.futimesBase = function (fd, atime, mtime) {
        var file = this.getFileByFdOrThrow(fd, 'futimes');
        var node = file.node;
        node.atime = new Date(atime * 1000);
        node.mtime = new Date(mtime * 1000);
    };
    Volume.prototype.futimesSync = function (fd, atime, mtime) {
        this.futimesBase(fd, toUnixTimestamp(atime), toUnixTimestamp(mtime));
    };
    Volume.prototype.futimes = function (fd, atime, mtime, callback) {
        this.wrapAsync(this.futimesBase, [fd, toUnixTimestamp(atime), toUnixTimestamp(mtime)], callback);
    };
    Volume.prototype.utimesBase = function (filename, atime, mtime) {
        var fd = this.openSync(filename, 'r');
        try {
            this.futimesBase(fd, atime, mtime);
        }
        finally {
            this.closeSync(fd);
        }
    };
    Volume.prototype.utimesSync = function (path, atime, mtime) {
        this.utimesBase(pathToFilename(path), toUnixTimestamp(atime), toUnixTimestamp(mtime));
    };
    Volume.prototype.utimes = function (path, atime, mtime, callback) {
        this.wrapAsync(this.utimesBase, [pathToFilename(path), toUnixTimestamp(atime), toUnixTimestamp(mtime)], callback);
    };
    Volume.prototype.mkdirBase = function (filename, modeNum) {
        var steps = filenameToSteps(filename);
        // This will throw if user tries to create root dir `fs.mkdirSync('/')`.
        if (!steps.length) {
            throw createError(EEXIST, 'mkdir', filename);
        }
        var dir = this.getLinkParentAsDirOrThrow(filename, 'mkdir');
        // Check path already exists.
        var name = steps[steps.length - 1];
        if (dir.getChild(name))
            throw createError(EEXIST, 'mkdir', filename);
        dir.createChild(name, this.createNode(true, modeNum));
    };
    /**
     * Creates directory tree recursively.
     * @param filename
     * @param modeNum
     */
    Volume.prototype.mkdirpBase = function (filename, modeNum) {
        var fullPath = resolve(filename);
        var fullPathSansSlash = fullPath.substring(1);
        var steps = !fullPathSansSlash ? [] : fullPathSansSlash.split(sep);
        var link = this.root;
        var created = false;
        for (var i = 0; i < steps.length; i++) {
            var step = steps[i];
            if (!link.getNode().isDirectory())
                throw createError(ENOTDIR, 'mkdir', link.getPath());
            var child = link.getChild(step);
            if (child) {
                if (child.getNode().isDirectory())
                    link = child;
                else
                    throw createError(ENOTDIR, 'mkdir', child.getPath());
            }
            else {
                link = link.createChild(step, this.createNode(true, modeNum));
                created = true;
            }
        }
        return created ? fullPath : undefined;
    };
    Volume.prototype.mkdirSync = function (path, options) {
        var opts = getMkdirOptions(options);
        var modeNum = modeToNumber(opts.mode, 511);
        var filename = pathToFilename(path);
        if (opts.recursive)
            return this.mkdirpBase(filename, modeNum);
        this.mkdirBase(filename, modeNum);
    };
    Volume.prototype.mkdir = function (path, a, b) {
        var opts = getMkdirOptions(a);
        var callback = validateCallback(typeof a === 'function' ? a : b);
        var modeNum = modeToNumber(opts.mode, 511);
        var filename = pathToFilename(path);
        if (opts.recursive)
            this.wrapAsync(this.mkdirpBase, [filename, modeNum], callback);
        else
            this.wrapAsync(this.mkdirBase, [filename, modeNum], callback);
    };
    // legacy interface
    Volume.prototype.mkdirpSync = function (path, mode) {
        return this.mkdirSync(path, { mode: mode, recursive: true });
    };
    Volume.prototype.mkdirp = function (path, a, b) {
        var mode = typeof a === 'function' ? undefined : a;
        var callback = validateCallback(typeof a === 'function' ? a : b);
        this.mkdir(path, { mode: mode, recursive: true }, callback);
    };
    Volume.prototype.mkdtempBase = function (prefix, encoding, retry) {
        if (retry === void 0) { retry = 5; }
        var filename = prefix + this.genRndStr();
        try {
            this.mkdirBase(filename, 511 /* MODE.DIR */);
            return (0, encoding_1.strToEncoding)(filename, encoding);
        }
        catch (err) {
            if (err.code === EEXIST) {
                if (retry > 1)
                    return this.mkdtempBase(prefix, encoding, retry - 1);
                else
                    throw Error('Could not create temp dir.');
            }
            else
                throw err;
        }
    };
    Volume.prototype.mkdtempSync = function (prefix, options) {
        var encoding = getDefaultOpts(options).encoding;
        if (!prefix || typeof prefix !== 'string')
            throw new TypeError('filename prefix is required');
        nullCheck(prefix);
        return this.mkdtempBase(prefix, encoding);
    };
    Volume.prototype.mkdtemp = function (prefix, a, b) {
        var _a = getDefaultOptsAndCb(a, b), encoding = _a[0].encoding, callback = _a[1];
        if (!prefix || typeof prefix !== 'string')
            throw new TypeError('filename prefix is required');
        if (!nullCheck(prefix))
            return;
        this.wrapAsync(this.mkdtempBase, [prefix, encoding], callback);
    };
    Volume.prototype.rmdirBase = function (filename, options) {
        var opts = getRmdirOptions(options);
        var link = this.getLinkAsDirOrThrow(filename, 'rmdir');
        // Check directory is empty.
        if (link.length && !opts.recursive)
            throw createError(ENOTEMPTY, 'rmdir', filename);
        this.deleteLink(link);
    };
    Volume.prototype.rmdirSync = function (path, options) {
        this.rmdirBase(pathToFilename(path), options);
    };
    Volume.prototype.rmdir = function (path, a, b) {
        var opts = getRmdirOptions(a);
        var callback = validateCallback(typeof a === 'function' ? a : b);
        this.wrapAsync(this.rmdirBase, [pathToFilename(path), opts], callback);
    };
    Volume.prototype.rmBase = function (filename, options) {
        if (options === void 0) { options = {}; }
        var link = this.getResolvedLink(filename);
        if (!link) {
            // "stat" is used to match Node's native error message.
            if (!options.force)
                throw createError(ENOENT, 'stat', filename);
            return;
        }
        if (link.getNode().isDirectory()) {
            if (!options.recursive) {
                throw createError(ERR_FS_EISDIR, 'rm', filename);
            }
        }
        this.deleteLink(link);
    };
    Volume.prototype.rmSync = function (path, options) {
        this.rmBase(pathToFilename(path), options);
    };
    Volume.prototype.rm = function (path, a, b) {
        var _a = getRmOptsAndCb(a, b), opts = _a[0], callback = _a[1];
        this.wrapAsync(this.rmBase, [pathToFilename(path), opts], callback);
    };
    Volume.prototype.fchmodBase = function (fd, modeNum) {
        var file = this.getFileByFdOrThrow(fd, 'fchmod');
        file.chmod(modeNum);
    };
    Volume.prototype.fchmodSync = function (fd, mode) {
        this.fchmodBase(fd, modeToNumber(mode));
    };
    Volume.prototype.fchmod = function (fd, mode, callback) {
        this.wrapAsync(this.fchmodBase, [fd, modeToNumber(mode)], callback);
    };
    Volume.prototype.chmodBase = function (filename, modeNum) {
        var fd = this.openSync(filename, 'r');
        try {
            this.fchmodBase(fd, modeNum);
        }
        finally {
            this.closeSync(fd);
        }
    };
    Volume.prototype.chmodSync = function (path, mode) {
        var modeNum = modeToNumber(mode);
        var filename = pathToFilename(path);
        this.chmodBase(filename, modeNum);
    };
    Volume.prototype.chmod = function (path, mode, callback) {
        var modeNum = modeToNumber(mode);
        var filename = pathToFilename(path);
        this.wrapAsync(this.chmodBase, [filename, modeNum], callback);
    };
    Volume.prototype.lchmodBase = function (filename, modeNum) {
        var fd = this.openBase(filename, O_RDWR, 0, false);
        try {
            this.fchmodBase(fd, modeNum);
        }
        finally {
            this.closeSync(fd);
        }
    };
    Volume.prototype.lchmodSync = function (path, mode) {
        var modeNum = modeToNumber(mode);
        var filename = pathToFilename(path);
        this.lchmodBase(filename, modeNum);
    };
    Volume.prototype.lchmod = function (path, mode, callback) {
        var modeNum = modeToNumber(mode);
        var filename = pathToFilename(path);
        this.wrapAsync(this.lchmodBase, [filename, modeNum], callback);
    };
    Volume.prototype.fchownBase = function (fd, uid, gid) {
        this.getFileByFdOrThrow(fd, 'fchown').chown(uid, gid);
    };
    Volume.prototype.fchownSync = function (fd, uid, gid) {
        validateUid(uid);
        validateGid(gid);
        this.fchownBase(fd, uid, gid);
    };
    Volume.prototype.fchown = function (fd, uid, gid, callback) {
        validateUid(uid);
        validateGid(gid);
        this.wrapAsync(this.fchownBase, [fd, uid, gid], callback);
    };
    Volume.prototype.chownBase = function (filename, uid, gid) {
        var link = this.getResolvedLinkOrThrow(filename, 'chown');
        var node = link.getNode();
        node.chown(uid, gid);
        // if(node.isFile() || node.isSymlink()) {
        //
        // } else if(node.isDirectory()) {
        //
        // } else {
        // TODO: What do we do here?
        // }
    };
    Volume.prototype.chownSync = function (path, uid, gid) {
        validateUid(uid);
        validateGid(gid);
        this.chownBase(pathToFilename(path), uid, gid);
    };
    Volume.prototype.chown = function (path, uid, gid, callback) {
        validateUid(uid);
        validateGid(gid);
        this.wrapAsync(this.chownBase, [pathToFilename(path), uid, gid], callback);
    };
    Volume.prototype.lchownBase = function (filename, uid, gid) {
        this.getLinkOrThrow(filename, 'lchown').getNode().chown(uid, gid);
    };
    Volume.prototype.lchownSync = function (path, uid, gid) {
        validateUid(uid);
        validateGid(gid);
        this.lchownBase(pathToFilename(path), uid, gid);
    };
    Volume.prototype.lchown = function (path, uid, gid, callback) {
        validateUid(uid);
        validateGid(gid);
        this.wrapAsync(this.lchownBase, [pathToFilename(path), uid, gid], callback);
    };
    Volume.prototype.watchFile = function (path, a, b) {
        var filename = pathToFilename(path);
        var options = a;
        var listener = b;
        if (typeof options === 'function') {
            listener = a;
            options = null;
        }
        if (typeof listener !== 'function') {
            throw Error('"watchFile()" requires a listener function');
        }
        var interval = 5007;
        var persistent = true;
        if (options && typeof options === 'object') {
            if (typeof options.interval === 'number')
                interval = options.interval;
            if (typeof options.persistent === 'boolean')
                persistent = options.persistent;
        }
        var watcher = this.statWatchers[filename];
        if (!watcher) {
            watcher = new this.StatWatcher();
            watcher.start(filename, persistent, interval);
            this.statWatchers[filename] = watcher;
        }
        watcher.addListener('change', listener);
        return watcher;
    };
    Volume.prototype.unwatchFile = function (path, listener) {
        var filename = pathToFilename(path);
        var watcher = this.statWatchers[filename];
        if (!watcher)
            return;
        if (typeof listener === 'function') {
            watcher.removeListener('change', listener);
        }
        else {
            watcher.removeAllListeners('change');
        }
        if (watcher.listenerCount('change') === 0) {
            watcher.stop();
            delete this.statWatchers[filename];
        }
    };
    Volume.prototype.createReadStream = function (path, options) {
        return new this.ReadStream(path, options);
    };
    Volume.prototype.createWriteStream = function (path, options) {
        return new this.WriteStream(path, options);
    };
    // watch(path: PathLike): FSWatcher;
    // watch(path: PathLike, options?: IWatchOptions | string): FSWatcher;
    Volume.prototype.watch = function (path, options, listener) {
        var filename = pathToFilename(path);
        var givenOptions = options;
        if (typeof options === 'function') {
            listener = options;
            givenOptions = null;
        }
        // tslint:disable-next-line prefer-const
        var _a = getDefaultOpts(givenOptions), persistent = _a.persistent, recursive = _a.recursive, encoding = _a.encoding;
        if (persistent === undefined)
            persistent = true;
        if (recursive === undefined)
            recursive = false;
        var watcher = new this.FSWatcher();
        watcher.start(filename, persistent, recursive, encoding);
        if (listener) {
            watcher.addListener('change', listener);
        }
        return watcher;
    };
    /**
     * Global file descriptor counter. UNIX file descriptors start from 0 and go sequentially
     * up, so here, in order not to conflict with them, we choose some big number and descrease
     * the file descriptor of every new opened file.
     * @type {number}
     * @todo This should not be static, right?
     */
    Volume.fd = 0x7fffffff;
    return Volume;
}());
exports.Volume = Volume;
function emitStop(self) {
    self.emit('stop');
}
var StatWatcher = /** @class */ (function (_super) {
    __extends(StatWatcher, _super);
    function StatWatcher(vol) {
        var _this = _super.call(this) || this;
        _this.onInterval = function () {
            try {
                var stats = _this.vol.statSync(_this.filename);
                if (_this.hasChanged(stats)) {
                    _this.emit('change', stats, _this.prev);
                    _this.prev = stats;
                }
            }
            finally {
                _this.loop();
            }
        };
        _this.vol = vol;
        return _this;
    }
    StatWatcher.prototype.loop = function () {
        this.timeoutRef = this.setTimeout(this.onInterval, this.interval);
    };
    StatWatcher.prototype.hasChanged = function (stats) {
        // if(!this.prev) return false;
        if (stats.mtimeMs > this.prev.mtimeMs)
            return true;
        if (stats.nlink !== this.prev.nlink)
            return true;
        return false;
    };
    StatWatcher.prototype.start = function (path, persistent, interval) {
        if (persistent === void 0) { persistent = true; }
        if (interval === void 0) { interval = 5007; }
        this.filename = pathToFilename(path);
        this.setTimeout = persistent
            ? setTimeout.bind(typeof globalThis !== 'undefined' ? globalThis : global)
            : setTimeoutUnref_1.default;
        this.interval = interval;
        this.prev = this.vol.statSync(this.filename);
        this.loop();
    };
    StatWatcher.prototype.stop = function () {
        clearTimeout(this.timeoutRef);
        process_1.default.nextTick(emitStop, this);
    };
    return StatWatcher;
}(events_1.EventEmitter));
exports.StatWatcher = StatWatcher;
var pool;
function allocNewPool(poolSize) {
    pool = (0, buffer_1.bufferAllocUnsafe)(poolSize);
    pool.used = 0;
}
util.inherits(FsReadStream, stream_1.Readable);
exports.ReadStream = FsReadStream;
function FsReadStream(vol, path, options) {
    if (!(this instanceof FsReadStream))
        return new FsReadStream(vol, path, options);
    this._vol = vol;
    // a little bit bigger buffer and water marks by default
    options = Object.assign({}, getOptions(options, {}));
    if (options.highWaterMark === undefined)
        options.highWaterMark = 64 * 1024;
    stream_1.Readable.call(this, options);
    this.path = pathToFilename(path);
    this.fd = options.fd === undefined ? null : options.fd;
    this.flags = options.flags === undefined ? 'r' : options.flags;
    this.mode = options.mode === undefined ? 438 : options.mode;
    this.start = options.start;
    this.end = options.end;
    this.autoClose = options.autoClose === undefined ? true : options.autoClose;
    this.pos = undefined;
    this.bytesRead = 0;
    if (this.start !== undefined) {
        if (typeof this.start !== 'number') {
            throw new TypeError('"start" option must be a Number');
        }
        if (this.end === undefined) {
            this.end = Infinity;
        }
        else if (typeof this.end !== 'number') {
            throw new TypeError('"end" option must be a Number');
        }
        if (this.start > this.end) {
            throw new Error('"start" option must be <= "end" option');
        }
        this.pos = this.start;
    }
    if (typeof this.fd !== 'number')
        this.open();
    this.on('end', function () {
        if (this.autoClose) {
            if (this.destroy)
                this.destroy();
        }
    });
}
FsReadStream.prototype.open = function () {
    var self = this; // tslint:disable-line no-this-assignment
    this._vol.open(this.path, this.flags, this.mode, function (er, fd) {
        if (er) {
            if (self.autoClose) {
                if (self.destroy)
                    self.destroy();
            }
            self.emit('error', er);
            return;
        }
        self.fd = fd;
        self.emit('open', fd);
        // start the flow of data.
        self.read();
    });
};
FsReadStream.prototype._read = function (n) {
    if (typeof this.fd !== 'number') {
        return this.once('open', function () {
            this._read(n);
        });
    }
    if (this.destroyed)
        return;
    if (!pool || pool.length - pool.used < kMinPoolSpace) {
        // discard the old pool.
        allocNewPool(this._readableState.highWaterMark);
    }
    // Grab another reference to the pool in the case that while we're
    // in the thread pool another read() finishes up the pool, and
    // allocates a new one.
    var thisPool = pool;
    var toRead = Math.min(pool.length - pool.used, n);
    var start = pool.used;
    if (this.pos !== undefined)
        toRead = Math.min(this.end - this.pos + 1, toRead);
    // already read everything we were supposed to read!
    // treat as EOF.
    if (toRead <= 0)
        return this.push(null);
    // the actual read.
    var self = this; // tslint:disable-line no-this-assignment
    this._vol.read(this.fd, pool, pool.used, toRead, this.pos, onread);
    // move the pool positions, and internal position for reading.
    if (this.pos !== undefined)
        this.pos += toRead;
    pool.used += toRead;
    function onread(er, bytesRead) {
        if (er) {
            if (self.autoClose && self.destroy) {
                self.destroy();
            }
            self.emit('error', er);
        }
        else {
            var b = null;
            if (bytesRead > 0) {
                self.bytesRead += bytesRead;
                b = thisPool.slice(start, start + bytesRead);
            }
            self.push(b);
        }
    }
};
FsReadStream.prototype._destroy = function (err, cb) {
    this.close(function (err2) {
        cb(err || err2);
    });
};
FsReadStream.prototype.close = function (cb) {
    var _this = this;
    var _a;
    if (cb)
        this.once('close', cb);
    if (this.closed || typeof this.fd !== 'number') {
        if (typeof this.fd !== 'number') {
            this.once('open', closeOnOpen);
            return;
        }
        return process_1.default.nextTick(function () { return _this.emit('close'); });
    }
    // Since Node 18, there is only a getter for '.closed'.
    // The first branch mimics other setters from Readable.
    // See https://github.com/nodejs/node/blob/v18.0.0/lib/internal/streams/readable.js#L1243
    if (typeof ((_a = this._readableState) === null || _a === void 0 ? void 0 : _a.closed) === 'boolean') {
        this._readableState.closed = true;
    }
    else {
        this.closed = true;
    }
    this._vol.close(this.fd, function (er) {
        if (er)
            _this.emit('error', er);
        else
            _this.emit('close');
    });
    this.fd = null;
};
// needed because as it will be called with arguments
// that does not match this.close() signature
function closeOnOpen(fd) {
    this.close();
}
util.inherits(FsWriteStream, stream_1.Writable);
exports.WriteStream = FsWriteStream;
function FsWriteStream(vol, path, options) {
    if (!(this instanceof FsWriteStream))
        return new FsWriteStream(vol, path, options);
    this._vol = vol;
    options = Object.assign({}, getOptions(options, {}));
    stream_1.Writable.call(this, options);
    this.path = pathToFilename(path);
    this.fd = options.fd === undefined ? null : options.fd;
    this.flags = options.flags === undefined ? 'w' : options.flags;
    this.mode = options.mode === undefined ? 438 : options.mode;
    this.start = options.start;
    this.autoClose = options.autoClose === undefined ? true : !!options.autoClose;
    this.pos = undefined;
    this.bytesWritten = 0;
    if (this.start !== undefined) {
        if (typeof this.start !== 'number') {
            throw new TypeError('"start" option must be a Number');
        }
        if (this.start < 0) {
            throw new Error('"start" must be >= zero');
        }
        this.pos = this.start;
    }
    if (options.encoding)
        this.setDefaultEncoding(options.encoding);
    if (typeof this.fd !== 'number')
        this.open();
    // dispose on finish.
    this.once('finish', function () {
        if (this.autoClose) {
            this.close();
        }
    });
}
FsWriteStream.prototype.open = function () {
    this._vol.open(this.path, this.flags, this.mode, function (er, fd) {
        if (er) {
            if (this.autoClose && this.destroy) {
                this.destroy();
            }
            this.emit('error', er);
            return;
        }
        this.fd = fd;
        this.emit('open', fd);
    }.bind(this));
};
FsWriteStream.prototype._write = function (data, encoding, cb) {
    if (!(data instanceof buffer_1.Buffer || data instanceof Uint8Array))
        return this.emit('error', new Error('Invalid data'));
    if (typeof this.fd !== 'number') {
        return this.once('open', function () {
            this._write(data, encoding, cb);
        });
    }
    var self = this; // tslint:disable-line no-this-assignment
    this._vol.write(this.fd, data, 0, data.length, this.pos, function (er, bytes) {
        if (er) {
            if (self.autoClose && self.destroy) {
                self.destroy();
            }
            return cb(er);
        }
        self.bytesWritten += bytes;
        cb();
    });
    if (this.pos !== undefined)
        this.pos += data.length;
};
FsWriteStream.prototype._writev = function (data, cb) {
    if (typeof this.fd !== 'number') {
        return this.once('open', function () {
            this._writev(data, cb);
        });
    }
    var self = this; // tslint:disable-line no-this-assignment
    var len = data.length;
    var chunks = new Array(len);
    var size = 0;
    for (var i = 0; i < len; i++) {
        var chunk = data[i].chunk;
        chunks[i] = chunk;
        size += chunk.length;
    }
    var buf = buffer_1.Buffer.concat(chunks);
    this._vol.write(this.fd, buf, 0, buf.length, this.pos, function (er, bytes) {
        if (er) {
            if (self.destroy)
                self.destroy();
            return cb(er);
        }
        self.bytesWritten += bytes;
        cb();
    });
    if (this.pos !== undefined)
        this.pos += size;
};
FsWriteStream.prototype.close = function (cb) {
    var _this = this;
    var _a;
    if (cb)
        this.once('close', cb);
    if (this.closed || typeof this.fd !== 'number') {
        if (typeof this.fd !== 'number') {
            this.once('open', closeOnOpen);
            return;
        }
        return process_1.default.nextTick(function () { return _this.emit('close'); });
    }
    // Since Node 18, there is only a getter for '.closed'.
    // The first branch mimics other setters from Writable.
    // See https://github.com/nodejs/node/blob/v18.0.0/lib/internal/streams/writable.js#L766
    if (typeof ((_a = this._writableState) === null || _a === void 0 ? void 0 : _a.closed) === 'boolean') {
        this._writableState.closed = true;
    }
    else {
        this.closed = true;
    }
    this._vol.close(this.fd, function (er) {
        if (er)
            _this.emit('error', er);
        else
            _this.emit('close');
    });
    this.fd = null;
};
FsWriteStream.prototype._destroy = FsReadStream.prototype._destroy;
// There is no shutdown() for files.
FsWriteStream.prototype.destroySoon = FsWriteStream.prototype.end;
// ---------------------------------------- FSWatcher
var FSWatcher = /** @class */ (function (_super) {
    __extends(FSWatcher, _super);
    function FSWatcher(vol) {
        var _this = _super.call(this) || this;
        _this._filename = '';
        _this._filenameEncoded = '';
        // _persistent: boolean = true;
        _this._recursive = false;
        _this._encoding = encoding_1.ENCODING_UTF8;
        // inode -> removers
        _this._listenerRemovers = new Map();
        _this._onParentChild = function (link) {
            if (link.getName() === _this._getName()) {
                _this._emit('rename');
            }
        };
        _this._emit = function (type) {
            _this.emit('change', type, _this._filenameEncoded);
        };
        _this._persist = function () {
            _this._timer = setTimeout(_this._persist, 1e6);
        };
        _this._vol = vol;
        return _this;
        // TODO: Emit "error" messages when watching.
        // this._handle.onchange = function(status, eventType, filename) {
        //     if (status < 0) {
        //         self._handle.close();
        //         const error = !filename ?
        //             errnoException(status, 'Error watching file for changes:') :
        //             errnoException(status, `Error watching file ${filename} for changes:`);
        //         error.filename = filename;
        //         self.emit('error', error);
        //     } else {
        //         self.emit('change', eventType, filename);
        //     }
        // };
    }
    FSWatcher.prototype._getName = function () {
        return this._steps[this._steps.length - 1];
    };
    FSWatcher.prototype.start = function (path, persistent, recursive, encoding) {
        var _this = this;
        if (persistent === void 0) { persistent = true; }
        if (recursive === void 0) { recursive = false; }
        if (encoding === void 0) { encoding = encoding_1.ENCODING_UTF8; }
        this._filename = pathToFilename(path);
        this._steps = filenameToSteps(this._filename);
        this._filenameEncoded = (0, encoding_1.strToEncoding)(this._filename);
        // this._persistent = persistent;
        this._recursive = recursive;
        this._encoding = encoding;
        try {
            this._link = this._vol.getLinkOrThrow(this._filename, 'FSWatcher');
        }
        catch (err) {
            var error = new Error("watch ".concat(this._filename, " ").concat(err.code));
            error.code = err.code;
            error.errno = err.code;
            throw error;
        }
        var watchLinkNodeChanged = function (link) {
            var _a;
            var filepath = link.getPath();
            var node = link.getNode();
            var onNodeChange = function () {
                var filename = relative(_this._filename, filepath);
                if (!filename) {
                    filename = _this._getName();
                }
                return _this.emit('change', 'change', filename);
            };
            node.on('change', onNodeChange);
            var removers = (_a = _this._listenerRemovers.get(node.ino)) !== null && _a !== void 0 ? _a : [];
            removers.push(function () { return node.removeListener('change', onNodeChange); });
            _this._listenerRemovers.set(node.ino, removers);
        };
        var watchLinkChildrenChanged = function (link) {
            var _a;
            var node = link.getNode();
            // when a new link added
            var onLinkChildAdd = function (l) {
                _this.emit('change', 'rename', relative(_this._filename, l.getPath()));
                setTimeout(function () {
                    // 1. watch changes of the new link-node
                    watchLinkNodeChanged(l);
                    // 2. watch changes of the new link-node's children
                    watchLinkChildrenChanged(l);
                });
            };
            // when a new link deleted
            var onLinkChildDelete = function (l) {
                // remove the listeners of the children nodes
                var removeLinkNodeListeners = function (curLink) {
                    var ino = curLink.getNode().ino;
                    var removers = _this._listenerRemovers.get(ino);
                    if (removers) {
                        removers.forEach(function (r) { return r(); });
                        _this._listenerRemovers.delete(ino);
                    }
                    Object.values(curLink.children).forEach(function (childLink) {
                        if (childLink) {
                            removeLinkNodeListeners(childLink);
                        }
                    });
                };
                removeLinkNodeListeners(l);
                _this.emit('change', 'rename', relative(_this._filename, l.getPath()));
            };
            // children nodes changed
            Object.entries(link.children).forEach(function (_a) {
                var name = _a[0], childLink = _a[1];
                if (childLink && name !== '.' && name !== '..') {
                    watchLinkNodeChanged(childLink);
                }
            });
            // link children add/remove
            link.on('child:add', onLinkChildAdd);
            link.on('child:delete', onLinkChildDelete);
            var removers = (_a = _this._listenerRemovers.get(node.ino)) !== null && _a !== void 0 ? _a : [];
            removers.push(function () {
                link.removeListener('child:add', onLinkChildAdd);
                link.removeListener('child:delete', onLinkChildDelete);
            });
            if (recursive) {
                Object.entries(link.children).forEach(function (_a) {
                    var name = _a[0], childLink = _a[1];
                    if (childLink && name !== '.' && name !== '..') {
                        watchLinkChildrenChanged(childLink);
                    }
                });
            }
        };
        watchLinkNodeChanged(this._link);
        watchLinkChildrenChanged(this._link);
        var parent = this._link.parent;
        if (parent) {
            // parent.on('child:add', this._onParentChild);
            parent.setMaxListeners(parent.getMaxListeners() + 1);
            parent.on('child:delete', this._onParentChild);
        }
        if (persistent)
            this._persist();
    };
    FSWatcher.prototype.close = function () {
        clearTimeout(this._timer);
        this._listenerRemovers.forEach(function (removers) {
            removers.forEach(function (r) { return r(); });
        });
        this._listenerRemovers.clear();
        var parent = this._link.parent;
        if (parent) {
            // parent.removeListener('child:add', this._onParentChild);
            parent.removeListener('child:delete', this._onParentChild);
        }
    };
    return FSWatcher;
}(events_1.EventEmitter));
exports.FSWatcher = FSWatcher;


/***/ }),

/***/ "../../node_modules/unzipit/dist/unzipit.module.js":
/*!*********************************************************!*\
  !*** ../../node_modules/unzipit/dist/unzipit.module.js ***!
  \*********************************************************/
/***/ ((module, __webpack_exports__, __webpack_require__) => {

"use strict";
__webpack_require__.r(__webpack_exports__);
/* harmony export */ __webpack_require__.d(__webpack_exports__, {
/* harmony export */   HTTPRangeReader: () => (/* binding */ HTTPRangeReader),
/* harmony export */   cleanup: () => (/* binding */ cleanup$1),
/* harmony export */   setOptions: () => (/* binding */ setOptions$1),
/* harmony export */   unzip: () => (/* binding */ unzip),
/* harmony export */   unzipRaw: () => (/* binding */ unzipRaw)
/* harmony export */ });
/* module decorator */ module = __webpack_require__.hmd(module);
/* unzipit@1.4.3, license MIT */
/* global SharedArrayBuffer, process */

function readBlobAsArrayBuffer(blob) {
  if (blob.arrayBuffer) {
    return blob.arrayBuffer();
  }
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.addEventListener('loadend', () => {
      resolve(reader.result);
    });
    reader.addEventListener('error', reject);
    reader.readAsArrayBuffer(blob);
  });
}

async function readBlobAsUint8Array(blob) {
  const arrayBuffer = await readBlobAsArrayBuffer(blob);
  return new Uint8Array(arrayBuffer);
}

function isBlob(v) {
  return typeof Blob !== 'undefined' && v instanceof Blob;
}

function isSharedArrayBuffer(b) {
  return typeof SharedArrayBuffer !== 'undefined' && b instanceof SharedArrayBuffer;
}

const isNode =
    (typeof process !== 'undefined') &&
    process.versions &&
    (typeof process.versions.node !== 'undefined') &&
    (typeof process.versions.electron === 'undefined');

function isTypedArraySameAsArrayBuffer(typedArray) {
  return typedArray.byteOffset === 0 && typedArray.byteLength === typedArray.buffer.byteLength;
}

class ArrayBufferReader {
  constructor(arrayBufferOrView) {
    this.typedArray = (arrayBufferOrView instanceof ArrayBuffer || isSharedArrayBuffer(arrayBufferOrView))
       ? new Uint8Array(arrayBufferOrView)
       : new Uint8Array(arrayBufferOrView.buffer, arrayBufferOrView.byteOffset, arrayBufferOrView.byteLength);
  }
  async getLength() {
    return this.typedArray.byteLength;
  }
  async read(offset, length) {
    return new Uint8Array(this.typedArray.buffer, this.typedArray.byteOffset + offset, length);
  }
}

class BlobReader {
  constructor(blob) {
    this.blob = blob;
  }
  async getLength() {
    return this.blob.size;
  }
  async read(offset, length) {
    const blob = this.blob.slice(offset, offset + length);
    const arrayBuffer = await readBlobAsArrayBuffer(blob);
    return new Uint8Array(arrayBuffer);
  }
  async sliceAsBlob(offset, length, type = '') {
    return this.blob.slice(offset, offset + length, type);
  }
}

class HTTPRangeReader {
  constructor(url) {
    this.url = url;
  }
  async getLength() {
    if (this.length === undefined) {
      const req = await fetch(this.url, { method: 'HEAD' });
      if (!req.ok) {
        throw new Error(`failed http request ${this.url}, status: ${req.status}: ${req.statusText}`);
      }
      this.length = parseInt(req.headers.get('content-length'));
      if (Number.isNaN(this.length)) {
        throw Error('could not get length');
      }
    }
    return this.length;
  }
  async read(offset, size) {
    if (size === 0) {
      return new Uint8Array(0);
    }
    const req = await fetch(this.url, {
      headers: {
        Range: `bytes=${offset}-${offset + size - 1}`,
      },
    });
    if (!req.ok) {
      throw new Error(`failed http request ${this.url}, status: ${req.status} offset: ${offset} size: ${size}: ${req.statusText}`);
    }
    const buffer = await req.arrayBuffer();
    return new Uint8Array(buffer);
  }
}

function inflate(data, buf) {
	var u8=Uint8Array;
	if(data[0]==3 && data[1]==0) return (buf ? buf : new u8(0));
	var bitsF = _bitsF, bitsE = _bitsE, decodeTiny = _decodeTiny, get17 = _get17;
	
	var noBuf = (buf==null);
	if(noBuf) buf = new u8((data.length>>>2)<<3);
	
	var BFINAL=0, BTYPE=0, HLIT=0, HDIST=0, HCLEN=0, ML=0, MD=0; 	
	var off = 0, pos = 0;
	var lmap, dmap;
	
	while(BFINAL==0) {		
		BFINAL = bitsF(data, pos  , 1);
		BTYPE  = bitsF(data, pos+1, 2);  pos+=3;
		//console.log(BFINAL, BTYPE);
		
		if(BTYPE==0) {
			if((pos&7)!=0) pos+=8-(pos&7);
			var p8 = (pos>>>3)+4, len = data[p8-4]|(data[p8-3]<<8);  //console.log(len);//bitsF(data, pos, 16), 
			if(noBuf) buf=_check(buf, off+len);
			buf.set(new u8(data.buffer, data.byteOffset+p8, len), off);
			//for(var i=0; i<len; i++) buf[off+i] = data[p8+i];
			//for(var i=0; i<len; i++) if(buf[off+i] != data[p8+i]) throw "e";
			pos = ((p8+len)<<3);  off+=len;  continue;
		}
		if(noBuf) buf=_check(buf, off+(1<<17));  // really not enough in many cases (but PNG and ZIP provide buffer in advance)
		if(BTYPE==1) {  lmap = U.flmap;  dmap = U.fdmap;  ML = (1<<9)-1;  MD = (1<<5)-1;   }
		if(BTYPE==2) {
			HLIT  = bitsE(data, pos   , 5)+257;  
			HDIST = bitsE(data, pos+ 5, 5)+  1;  
			HCLEN = bitsE(data, pos+10, 4)+  4;  pos+=14;
			for(var i=0; i<38; i+=2) {  U.itree[i]=0;  U.itree[i+1]=0;  }
			var tl = 1;
			for(var i=0; i<HCLEN; i++) {  var l=bitsE(data, pos+i*3, 3);  U.itree[(U.ordr[i]<<1)+1] = l;  if(l>tl)tl=l;  }     pos+=3*HCLEN;  //console.log(itree);
			makeCodes(U.itree, tl);
			codes2map(U.itree, tl, U.imap);
			
			lmap = U.lmap;  dmap = U.dmap;
			
			pos = decodeTiny(U.imap, (1<<tl)-1, HLIT+HDIST, data, pos, U.ttree);
			var mx0 = _copyOut(U.ttree,    0, HLIT , U.ltree);  ML = (1<<mx0)-1;
			var mx1 = _copyOut(U.ttree, HLIT, HDIST, U.dtree);  MD = (1<<mx1)-1;
			
			//var ml = decodeTiny(U.imap, (1<<tl)-1, HLIT , data, pos, U.ltree); ML = (1<<(ml>>>24))-1;  pos+=(ml&0xffffff);
			makeCodes(U.ltree, mx0);
			codes2map(U.ltree, mx0, lmap);
			
			//var md = decodeTiny(U.imap, (1<<tl)-1, HDIST, data, pos, U.dtree); MD = (1<<(md>>>24))-1;  pos+=(md&0xffffff);
			makeCodes(U.dtree, mx1);
			codes2map(U.dtree, mx1, dmap);
		}
		//var ooff=off, opos=pos;
		while(true) {
			var code = lmap[get17(data, pos) & ML];  pos += code&15;
			var lit = code>>>4;  //U.lhst[lit]++;  
			if((lit>>>8)==0) {  buf[off++] = lit;  }
			else if(lit==256) {  break;  }
			else {
				var end = off+lit-254;
				if(lit>264) { var ebs = U.ldef[lit-257];  end = off + (ebs>>>3) + bitsE(data, pos, ebs&7);  pos += ebs&7;  }
				//dst[end-off]++;
				
				var dcode = dmap[get17(data, pos) & MD];  pos += dcode&15;
				var dlit = dcode>>>4;
				var dbs = U.ddef[dlit], dst = (dbs>>>4) + bitsF(data, pos, dbs&15);  pos += dbs&15;
				
				//var o0 = off-dst, stp = Math.min(end-off, dst);
				//if(stp>20) while(off<end) {  buf.copyWithin(off, o0, o0+stp);  off+=stp;  }  else
				//if(end-dst<=off) buf.copyWithin(off, off-dst, end-dst);  else
				//if(dst==1) buf.fill(buf[off-1], off, end);  else
				if(noBuf) buf=_check(buf, off+(1<<17));
				while(off<end) {  buf[off]=buf[off++-dst];    buf[off]=buf[off++-dst];  buf[off]=buf[off++-dst];  buf[off]=buf[off++-dst];  }   
				off=end;
				//while(off!=end) {  buf[off]=buf[off++-dst];  }
			}
		}
		//console.log(off-ooff, (pos-opos)>>>3);
	}
	//console.log(dst);
	//console.log(tlen, dlen, off-tlen+tcnt);
	return buf.length==off ? buf : buf.slice(0,off);
}
function _check(buf, len) {
	var bl=buf.length;  if(len<=bl) return buf;
	var nbuf = new Uint8Array(Math.max(bl<<1,len));  nbuf.set(buf,0);
	//for(var i=0; i<bl; i+=4) {  nbuf[i]=buf[i];  nbuf[i+1]=buf[i+1];  nbuf[i+2]=buf[i+2];  nbuf[i+3]=buf[i+3];  }
	return nbuf;
}

function _decodeTiny(lmap, LL, len, data, pos, tree) {
	var bitsE = _bitsE, get17 = _get17;
	var i = 0;
	while(i<len) {
		var code = lmap[get17(data, pos)&LL];  pos+=code&15;
		var lit = code>>>4; 
		if(lit<=15) {  tree[i]=lit;  i++;  }
		else {
			var ll = 0, n = 0;
			if(lit==16) {
				n = (3  + bitsE(data, pos, 2));  pos += 2;  ll = tree[i-1];
			}
			else if(lit==17) {
				n = (3  + bitsE(data, pos, 3));  pos += 3;
			}
			else if(lit==18) {
				n = (11 + bitsE(data, pos, 7));  pos += 7;
			}
			var ni = i+n;
			while(i<ni) {  tree[i]=ll;  i++; }
		}
	}
	return pos;
}
function _copyOut(src, off, len, tree) {
	var mx=0, i=0, tl=tree.length>>>1;
	while(i<len) {  var v=src[i+off];  tree[(i<<1)]=0;  tree[(i<<1)+1]=v;  if(v>mx)mx=v;  i++;  }
	while(i<tl ) {  tree[(i<<1)]=0;  tree[(i<<1)+1]=0;  i++;  }
	return mx;
}

function makeCodes(tree, MAX_BITS) {  // code, length
	var max_code = tree.length;
	var code, bits, n, i, len;
	
	var bl_count = U.bl_count;  for(var i=0; i<=MAX_BITS; i++) bl_count[i]=0;
	for(i=1; i<max_code; i+=2) bl_count[tree[i]]++;
	
	var next_code = U.next_code;	// smallest code for each length
	
	code = 0;
	bl_count[0] = 0;
	for (bits = 1; bits <= MAX_BITS; bits++) {
		code = (code + bl_count[bits-1]) << 1;
		next_code[bits] = code;
	}
	
	for (n = 0; n < max_code; n+=2) {
		len = tree[n+1];
		if (len != 0) {
			tree[n] = next_code[len];
			next_code[len]++;
		}
	}
}
function codes2map(tree, MAX_BITS, map) {
	var max_code = tree.length;
	var r15 = U.rev15;
	for(var i=0; i<max_code; i+=2) if(tree[i+1]!=0)  {
		var lit = i>>1;
		var cl = tree[i+1], val = (lit<<4)|cl; // :  (0x8000 | (U.of0[lit-257]<<7) | (U.exb[lit-257]<<4) | cl);
		var rest = (MAX_BITS-cl), i0 = tree[i]<<rest, i1 = i0 + (1<<rest);
		//tree[i]=r15[i0]>>>(15-MAX_BITS);
		while(i0!=i1) {
			var p0 = r15[i0]>>>(15-MAX_BITS);
			map[p0]=val;  i0++;
		}
	}
}
function revCodes(tree, MAX_BITS) {
	var r15 = U.rev15, imb = 15-MAX_BITS;
	for(var i=0; i<tree.length; i+=2) {  var i0 = (tree[i]<<(MAX_BITS-tree[i+1]));  tree[i] = r15[i0]>>>imb;  }
}

function _bitsE(dt, pos, length) {  return ((dt[pos>>>3] | (dt[(pos>>>3)+1]<<8)                        )>>>(pos&7))&((1<<length)-1);  }
function _bitsF(dt, pos, length) {  return ((dt[pos>>>3] | (dt[(pos>>>3)+1]<<8) | (dt[(pos>>>3)+2]<<16))>>>(pos&7))&((1<<length)-1);  }
/*
function _get9(dt, pos) {
	return ((dt[pos>>>3] | (dt[(pos>>>3)+1]<<8))>>>(pos&7))&511;
} */
function _get17(dt, pos) {	// return at least 17 meaningful bytes
	return (dt[pos>>>3] | (dt[(pos>>>3)+1]<<8) | (dt[(pos>>>3)+2]<<16) )>>>(pos&7);
}
const U = function(){
	var u16=Uint16Array, u32=Uint32Array;
	return {
		next_code : new u16(16),
		bl_count  : new u16(16),
		ordr : [ 16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15 ],
		of0  : [3,4,5,6,7,8,9,10,11,13,15,17,19,23,27,31,35,43,51,59,67,83,99,115,131,163,195,227,258,999,999,999],
		exb  : [0,0,0,0,0,0,0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4,  4,  5,  5,  5,  5,  0,  0,  0,  0],
		ldef : new u16(32),
		df0  : [1,2,3,4,5,7,9,13,17,25,33,49,65,97,129,193,257,385,513,769,1025,1537,2049,3073,4097,6145,8193,12289,16385,24577, 65535, 65535],
		dxb  : [0,0,0,0,1,1,2, 2, 3, 3, 4, 4, 5, 5,  6,  6,  7,  7,  8,  8,   9,   9,  10,  10,  11,  11,  12,   12,   13,   13,     0,     0],
		ddef : new u32(32),
		flmap: new u16(  512),  fltree: [],
		fdmap: new u16(   32),  fdtree: [],
		lmap : new u16(32768),  ltree : [],  ttree:[],
		dmap : new u16(32768),  dtree : [],
		imap : new u16(  512),  itree : [],
		//rev9 : new u16(  512)
		rev15: new u16(1<<15),
		lhst : new u32(286), dhst : new u32( 30), ihst : new u32(19),
		lits : new u32(15000),
		strt : new u16(1<<16),
		prev : new u16(1<<15)
	};  
} ();

(function(){	
	var len = 1<<15;
	for(var i=0; i<len; i++) {
		var x = i;
		x = (((x & 0xaaaaaaaa) >>> 1) | ((x & 0x55555555) << 1));
		x = (((x & 0xcccccccc) >>> 2) | ((x & 0x33333333) << 2));
		x = (((x & 0xf0f0f0f0) >>> 4) | ((x & 0x0f0f0f0f) << 4));
		x = (((x & 0xff00ff00) >>> 8) | ((x & 0x00ff00ff) << 8));
		U.rev15[i] = (((x >>> 16) | (x << 16)))>>>17;
	}
	
	function pushV(tgt, n, sv) {  while(n--!=0) tgt.push(0,sv);  }
	
	for(var i=0; i<32; i++) {  U.ldef[i]=(U.of0[i]<<3)|U.exb[i];  U.ddef[i]=(U.df0[i]<<4)|U.dxb[i];  }
	
	pushV(U.fltree, 144, 8);  pushV(U.fltree, 255-143, 9);  pushV(U.fltree, 279-255, 7);  pushV(U.fltree,287-279,8);
	/*
	var i = 0;
	for(; i<=143; i++) U.fltree.push(0,8);
	for(; i<=255; i++) U.fltree.push(0,9);
	for(; i<=279; i++) U.fltree.push(0,7);
	for(; i<=287; i++) U.fltree.push(0,8);
	*/
	makeCodes(U.fltree, 9);
	codes2map(U.fltree, 9, U.flmap);
	revCodes (U.fltree, 9);
	
	pushV(U.fdtree,32,5);
	//for(i=0;i<32; i++) U.fdtree.push(0,5);
	makeCodes(U.fdtree, 5);
	codes2map(U.fdtree, 5, U.fdmap);
	revCodes (U.fdtree, 5);
	
	pushV(U.itree,19,0);  pushV(U.ltree,286,0);  pushV(U.dtree,30,0);  pushV(U.ttree,320,0);
	/*
	for(var i=0; i< 19; i++) U.itree.push(0,0);
	for(var i=0; i<286; i++) U.ltree.push(0,0);
	for(var i=0; i< 30; i++) U.dtree.push(0,0);
	for(var i=0; i<320; i++) U.ttree.push(0,0);
	*/
})();

const crc = {
	table : ( function() {
	   var tab = new Uint32Array(256);
	   for (var n=0; n<256; n++) {
			var c = n;
			for (var k=0; k<8; k++) {
				if (c & 1)  c = 0xedb88320 ^ (c >>> 1);
				else        c = c >>> 1;
			}
			tab[n] = c;  }    
		return tab;  })(),
	update : function(c, buf, off, len) {
		for (var i=0; i<len; i++)  c = crc.table[(c ^ buf[off+i]) & 0xff] ^ (c >>> 8);
		return c;
	},
	crc : function(b,o,l)  {  return crc.update(0xffffffff,b,o,l) ^ 0xffffffff;  }
};

function inflateRaw(file, buf) {  return inflate(file, buf);  }

/* global module */

const config = {
  numWorkers: 1,
  workerURL: '',
  useWorkers: false,
};

let nextId = 0;

// Requests are put on a queue.
// We don't send the request to the worker until the worker
// is finished. This probably adds a small amount of latency
// but the issue is imagine you have 2 workers. You give worker
// A x seconds of work to do and worker B y seconds of work to
// do. You don't know which will finish first. If you give
// the worker with more work to do the request then you'll
// waste time.

// note: we can't check `workers.length` for deciding if
// we've reached `config.numWorkers` because creation the worker
// is async which means other requests to make workers might
// come in before a worker gets added to `workers`
let numWorkers = 0;
let canUseWorkers = true;   // gets set to false if we can't start a worker
const workers = [];
const availableWorkers = [];
const waitingForWorkerQueue = [];
const currentlyProcessingIdToRequestMap = new Map();

function handleResult(e) {
  makeWorkerAvailable(e.target);
  const {id, error, data} = e.data;
  const request = currentlyProcessingIdToRequestMap.get(id);
  currentlyProcessingIdToRequestMap.delete(id);
  if (error) {
    request.reject(error);
  } else {
    request.resolve(data);
  }
}

// Because Firefox uses non-standard onerror to signal an error.
function startWorker(url) {
  return new Promise((resolve, reject) => {
    const worker = new Worker(url);
    worker.onmessage = (e) => {
      if (e.data === 'start') {
        worker.onerror = undefined;
        worker.onmessage = undefined;
        resolve(worker);
      } else {
        reject(new Error(`unexpected message: ${e.data}`));
      }
    };
    worker.onerror = reject;
  });
}

function dynamicRequire(mod, request) {
  return mod.require ? mod.require(request) : {};
}

const workerHelper = (function() {
  if (isNode) {
    // We need to use `dynamicRequire` because `require` on it's own will be optimized by webpack.
    const {Worker} = dynamicRequire(module, 'worker_threads');
    return {
      async createWorker(url) {
        return new Worker(url);
      },
      addEventListener(worker, fn) {
        worker.on('message', (data) => {
          fn({target: worker, data});
        });
      },
      async terminate(worker) {
        await worker.terminate();
      },
    };
  } else {
    return {
      async createWorker(url) {
        // I don't understand this security issue
        // Apparently there is some iframe setting or http header
        // that prevents cross domain workers. But, I can manually
        // download the text and do it. I reported this to Chrome
        // and they said it was fine so ¯\_(ツ)_/¯
        try {
          const worker = await startWorker(url);
          return worker;
        } catch (e) {
          console.warn('could not load worker:', url);
        }

        let text;
        try {
          const req = await fetch(url, {mode: 'cors'});
          if (!req.ok) {
            throw new Error(`could not load: ${url}`);
          }
          text = await req.text();
          url = URL.createObjectURL(new Blob([text], {type: 'application/javascript'}));
          const worker = await startWorker(url);
          config.workerURL = url;  // this is a hack. What's a better way to structure this code?
          return worker;
        } catch (e) {
          console.warn('could not load worker via fetch:', url);
        }

        if (text !== undefined) {
          try {
            url = `data:application/javascript;base64,${btoa(text)}`;
            const worker = await startWorker(url);
            config.workerURL = url;
            return worker;
          } catch (e) {
            console.warn('could not load worker via dataURI');
          }
        }

        console.warn('workers will not be used');
        throw new Error('can not start workers');
      },
      addEventListener(worker, fn) {
        worker.addEventListener('message', fn);
      },
      async terminate(worker) {
        worker.terminate();
      },
    };
  }
}());

function makeWorkerAvailable(worker) {
  availableWorkers.push(worker);
  processWaitingForWorkerQueue();
}

async function getAvailableWorker() {
  if (availableWorkers.length === 0 && numWorkers < config.numWorkers) {
    ++numWorkers;  // see comment at numWorkers declaration
    try {
      const worker = await workerHelper.createWorker(config.workerURL);
      workers.push(worker);
      availableWorkers.push(worker);
      workerHelper.addEventListener(worker, handleResult);
    } catch (e) {
      // set this global out-of-band (needs refactor)
      canUseWorkers = false;
    }
  }
  return availableWorkers.pop();
}

// @param {Uint8Array} src
// @param {number} uncompressedSize
// @param {string} [type] mime-type
// @returns {ArrayBuffer|Blob} ArrayBuffer if type is falsy or Blob otherwise.
function inflateRawLocal(src, uncompressedSize, type, resolve) {
  const dst = new Uint8Array(uncompressedSize);
  inflateRaw(src, dst);
  resolve(type
     ? new Blob([dst], {type})
     : dst.buffer);
}

async function processWaitingForWorkerQueue() {
  if (waitingForWorkerQueue.length === 0) {
    return;
  }

  if (config.useWorkers && canUseWorkers) {
    const worker = await getAvailableWorker();
    // canUseWorkers might have been set out-of-band (need refactor)
    if (canUseWorkers) {
      if (worker) {
        if (waitingForWorkerQueue.length === 0) {
          // the queue might be empty while we awaited for a worker.
          makeWorkerAvailable(worker);
          return;
        }
        const {id, src, uncompressedSize, type, resolve, reject} = waitingForWorkerQueue.shift();
        currentlyProcessingIdToRequestMap.set(id, {id, resolve, reject});
        const transferables = [];
        // NOTE: Originally I thought you could transfer an ArrayBuffer.
        // The code on this side is often using views into the entire file
        // which means if we transferred we'd lose the entire file. That sucks
        // because it means there's an expensive copy to send the uncompressed
        // data to the worker.
        //
        // Also originally I thought we could send a Blob but we'd need to refactor
        // the code in unzipit/readEntryData as currently it reads the uncompressed
        // bytes.
        //
        //if (!isBlob(src) && !isSharedArrayBuffer(src)) {
        //  transferables.push(src);
        //}
        worker.postMessage({
          type: 'inflate',
          data: {
            id,
            type,
            src,
            uncompressedSize,
          },
        }, transferables);
      }
      return;
    }
  }

  // inflate locally
  // We loop here because what happens if many requests happen at once
  // the first N requests will try to async make a worker. Other requests
  // will then be on the queue. But if we fail to make workers then there
  // are pending requests.
  while (waitingForWorkerQueue.length) {
    const {src, uncompressedSize, type, resolve} = waitingForWorkerQueue.shift();
    let data = src;
    if (isBlob(src)) {
      data = await readBlobAsUint8Array(src);
    }
    inflateRawLocal(data, uncompressedSize, type, resolve);
  }
}

function setOptions(options) {
  config.workerURL = options.workerURL || config.workerURL;
  // there's no reason to set the workerURL if you're not going to use workers
  if (options.workerURL) {
    config.useWorkers = true;
  }
  config.useWorkers = options.useWorkers !== undefined ? options.useWorkers : config.useWorkers;
  config.numWorkers = options.numWorkers || config.numWorkers;
}

// It has to take non-zero time to put a large typed array in a Blob since the very
// next instruction you could change the contents of the array. So, if you're reading
// the zip file for images/video/audio then all you want is a Blob on which to get a URL.
// so that operation of putting the data in a Blob should happen in the worker.
//
// Conversely if you want the data itself then you want an ArrayBuffer immediately
// since the worker can transfer its ArrayBuffer zero copy.
//
// @param {Uint8Array|Blob} src
// @param {number} uncompressedSize
// @param {string} [type] falsy or mimeType string (eg: 'image/png')
// @returns {ArrayBuffer|Blob} ArrayBuffer if type is falsy or Blob otherwise.
function inflateRawAsync(src, uncompressedSize, type) {
  return new Promise((resolve, reject) => {
    // note: there is potential an expensive copy here. In order for the data
    // to make it into the worker we need to copy the data to the worker unless
    // it's a Blob or a SharedArrayBuffer.
    //
    // Solutions:
    //
    // 1. A minor enhancement, if `uncompressedSize` is small don't call the worker.
    //
    //    might be a win period as their is overhead calling the worker
    //
    // 2. Move the entire library to the worker
    //
    //    Good, Maybe faster if you pass a URL, Blob, or SharedArrayBuffer? Not sure about that
    //    as those are also easy to transfer. Still slow if you pass an ArrayBuffer
    //    as the ArrayBuffer has to be copied to the worker.
    //
    // I guess benchmarking is really the only thing to try.
    waitingForWorkerQueue.push({src, uncompressedSize, type, resolve, reject, id: nextId++});
    processWaitingForWorkerQueue();
  });
}

function clearArray(arr) {
  arr.splice(0, arr.length);
}

async function cleanup() {
  for (const worker of workers) {
    await workerHelper.terminate(worker);
  }
  clearArray(workers);
  clearArray(availableWorkers);
  clearArray(waitingForWorkerQueue);
  currentlyProcessingIdToRequestMap.clear();
  numWorkers = 0;
  canUseWorkers = true;
}

/*
class Zip {
  constructor(reader) {
    comment,  // the comment for this entry
    commentBytes, // the raw comment for this entry
  }
}
*/

function dosDateTimeToDate(date, time) {
  const day = date & 0x1f; // 1-31
  const month = (date >> 5 & 0xf) - 1; // 1-12, 0-11
  const year = (date >> 9 & 0x7f) + 1980; // 0-128, 1980-2108

  const millisecond = 0;
  const second = (time & 0x1f) * 2; // 0-29, 0-58 (even numbers)
  const minute = time >> 5 & 0x3f; // 0-59
  const hour = time >> 11 & 0x1f; // 0-23

  return new Date(year, month, day, hour, minute, second, millisecond);
}

class ZipEntry {
  constructor(reader, rawEntry) {
    this._reader = reader;
    this._rawEntry = rawEntry;
    this.name = rawEntry.name;
    this.nameBytes = rawEntry.nameBytes;
    this.size = rawEntry.uncompressedSize;
    this.compressedSize = rawEntry.compressedSize;
    this.comment = rawEntry.comment;
    this.commentBytes = rawEntry.commentBytes;
    this.compressionMethod = rawEntry.compressionMethod;
    this.lastModDate = dosDateTimeToDate(rawEntry.lastModFileDate, rawEntry.lastModFileTime);
    this.isDirectory = rawEntry.uncompressedSize === 0 && rawEntry.name.endsWith('/');
    this.encrypted = !!(rawEntry.generalPurposeBitFlag & 0x1);
    this.externalFileAttributes = rawEntry.externalFileAttributes;
    this.versionMadeBy = rawEntry.versionMadeBy;
  }
  // returns a promise that returns a Blob for this entry
  async blob(type = 'application/octet-stream') {
    return await readEntryDataAsBlob(this._reader, this._rawEntry, type);
  }
  // returns a promise that returns an ArrayBuffer for this entry
  async arrayBuffer() {
    return await readEntryDataAsArrayBuffer(this._reader, this._rawEntry);
  }
  // returns text, assumes the text is valid utf8. If you want more options decode arrayBuffer yourself
  async text() {
    const buffer = await this.arrayBuffer();
    return decodeBuffer(new Uint8Array(buffer));
  }
  // returns text with JSON.parse called on it. If you want more options decode arrayBuffer yourself
  async json() {
    const text = await this.text();
    return JSON.parse(text);
  }
}

const EOCDR_WITHOUT_COMMENT_SIZE = 22;
const MAX_COMMENT_SIZE = 0xffff; // 2-byte size
const EOCDR_SIGNATURE = 0x06054b50;
const ZIP64_EOCDR_SIGNATURE = 0x06064b50;

async function readAs(reader, offset, length) {
  return await reader.read(offset, length);
}

// The point of this function is we want to be able to pass the data
// to a worker as fast as possible so when decompressing if the data
// is already a blob and we can get a blob then get a blob.
//
// I'm not sure what a better way to refactor this is. We've got examples
// of multiple readers. Ideally, for every type of reader we could ask
// it, "give me a type that is zero copy both locally and when sent to a worker".
//
// The problem is the worker would also have to know the how to handle this
// opaque type. I suppose the correct solution is to register different
// reader handlers in the worker so BlobReader would register some
// `handleZeroCopyType<BlobReader>`. At the moment I don't feel like
// refactoring. As it is you just pass in an instance of the reader
// but instead you'd have to register the reader and some how get the
// source for the `handleZeroCopyType` handler function into the worker.
// That sounds like a huge PITA, requiring you to put the implementation
// in a separate file so the worker can load it or some other workaround
// hack.
//
// For now this hack works even if it's not generic.
async function readAsBlobOrTypedArray(reader, offset, length, type) {
  if (reader.sliceAsBlob) {
    return await reader.sliceAsBlob(offset, length, type);
  }
  return await reader.read(offset, length);
}

const crc$1 = {
  unsigned() {
    return 0;
  },
};

function getUint16LE(uint8View, offset) {
  return uint8View[offset    ] +
         uint8View[offset + 1] * 0x100;
}

function getUint32LE(uint8View, offset) {
  return uint8View[offset    ] +
         uint8View[offset + 1] * 0x100 +
         uint8View[offset + 2] * 0x10000 +
         uint8View[offset + 3] * 0x1000000;
}

function getUint64LE(uint8View, offset) {
  return getUint32LE(uint8View, offset) +
         getUint32LE(uint8View, offset + 4) * 0x100000000;
}

/* eslint-disable no-irregular-whitespace */
// const decodeCP437 = (function() {
//   const cp437 = '\u0000☺☻♥♦♣♠•◘○◙♂♀♪♫☼►◄↕‼¶§▬↨↑↓→←∟↔▲▼ !"#$%&\'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~⌂ÇüéâäàåçêëèïîìÄÅÉæÆôöòûùÿÖÜ¢£¥₧ƒáíóúñÑªº¿⌐¬½¼¡«»░▒▓│┤╡╢╖╕╣║╗╝╜╛┐└┴┬├─┼╞╟╚╔╩╦╠═╬╧╨╤╥╙╘╒╓╫╪┘┌█▄▌▐▀αßΓπΣσµτΦΘΩδ∞φε∩≡±≥≤⌠⌡÷≈°∙·√ⁿ²■ ';
//
//   return function(uint8view) {
//     return Array.from(uint8view).map(v => cp437[v]).join('');
//   };
// }());
/* eslint-enable no-irregular-whitespace */

const utf8Decoder = new TextDecoder();
function decodeBuffer(uint8View, isUTF8) {  /* eslint-disable-line no-unused-vars */ /* lgtm [js/superfluous-trailing-arguments] */
  if (isSharedArrayBuffer(uint8View.buffer)) {
    uint8View = new Uint8Array(uint8View);
  }
  return utf8Decoder.decode(uint8View);
  /*
  AFAICT the UTF8 flat is not set so it's 100% up to the user
  to self decode if their file is not utf8 filenames
  return isUTF8
      ? utf8Decoder.decode(uint8View)
      : decodeCP437(uint8View);
  */
}

async function findEndOfCentralDirector(reader, totalLength) {
  const size = Math.min(EOCDR_WITHOUT_COMMENT_SIZE + MAX_COMMENT_SIZE, totalLength);
  const readStart = totalLength - size;
  const data = await readAs(reader, readStart, size);
  for (let i = size - EOCDR_WITHOUT_COMMENT_SIZE; i >= 0; --i) {
    if (getUint32LE(data, i) !== EOCDR_SIGNATURE) {
      continue;
    }

    // 0 - End of central directory signature
    const eocdr = new Uint8Array(data.buffer, data.byteOffset + i, data.byteLength - i);
    // 4 - Number of this disk
    const diskNumber = getUint16LE(eocdr, 4);
    if (diskNumber !== 0) {
      throw new Error(`multi-volume zip files are not supported. This is volume: ${diskNumber}`);
    }

    // 6 - Disk where central directory starts
    // 8 - Number of central directory records on this disk
    // 10 - Total number of central directory records
    const entryCount = getUint16LE(eocdr, 10);
    // 12 - Size of central directory (bytes)
    const centralDirectorySize = getUint32LE(eocdr, 12);
    // 16 - Offset of start of central directory, relative to start of archive
    const centralDirectoryOffset = getUint32LE(eocdr, 16);
    // 20 - Comment length
    const commentLength = getUint16LE(eocdr, 20);
    const expectedCommentLength = eocdr.length - EOCDR_WITHOUT_COMMENT_SIZE;
    if (commentLength !== expectedCommentLength) {
      throw new Error(`invalid comment length. expected: ${expectedCommentLength}, actual: ${commentLength}`);
    }

    // 22 - Comment
    // the encoding is always cp437.
    const commentBytes = new Uint8Array(eocdr.buffer, eocdr.byteOffset + 22, commentLength);
    const comment = decodeBuffer(commentBytes);

    if (entryCount === 0xffff || centralDirectoryOffset === 0xffffffff) {
      return await readZip64CentralDirectory(reader, readStart + i, comment, commentBytes);
    } else {
      return await readEntries(reader, centralDirectoryOffset, centralDirectorySize, entryCount, comment, commentBytes);
    }
  }

  throw new Error('could not find end of central directory. maybe not zip file');
}

const END_OF_CENTRAL_DIRECTORY_LOCATOR_SIGNATURE = 0x07064b50;

async function readZip64CentralDirectory(reader, offset, comment, commentBytes) {
  // ZIP64 Zip64 end of central directory locator
  const zip64EocdlOffset = offset - 20;
  const eocdl = await readAs(reader, zip64EocdlOffset, 20);

  // 0 - zip64 end of central dir locator signature
  if (getUint32LE(eocdl, 0) !== END_OF_CENTRAL_DIRECTORY_LOCATOR_SIGNATURE) {
    throw new Error('invalid zip64 end of central directory locator signature');
  }

  // 4 - number of the disk with the start of the zip64 end of central directory
  // 8 - relative offset of the zip64 end of central directory record
  const zip64EocdrOffset = getUint64LE(eocdl, 8);
  // 16 - total number of disks

  // ZIP64 end of central directory record
  const zip64Eocdr = await readAs(reader, zip64EocdrOffset, 56);

  // 0 - zip64 end of central dir signature                           4 bytes  (0x06064b50)
  if (getUint32LE(zip64Eocdr, 0) !== ZIP64_EOCDR_SIGNATURE) {
    throw new Error('invalid zip64 end of central directory record signature');
  }
  // 4 - size of zip64 end of central directory record                8 bytes
  // 12 - version made by                                             2 bytes
  // 14 - version needed to extract                                   2 bytes
  // 16 - number of this disk                                         4 bytes
  // 20 - number of the disk with the start of the central directory  4 bytes
  // 24 - total number of entries in the central directory on this disk         8 bytes
  // 32 - total number of entries in the central directory            8 bytes
  const entryCount = getUint64LE(zip64Eocdr, 32);
  // 40 - size of the central directory                               8 bytes
  const centralDirectorySize = getUint64LE(zip64Eocdr, 40);
  // 48 - offset of start of central directory with respect to the starting disk number     8 bytes
  const centralDirectoryOffset = getUint64LE(zip64Eocdr, 48);
  // 56 - zip64 extensible data sector                                (variable size)
  return readEntries(reader, centralDirectoryOffset, centralDirectorySize, entryCount, comment, commentBytes);
}

const CENTRAL_DIRECTORY_FILE_HEADER_SIGNATURE = 0x02014b50;

async function readEntries(reader, centralDirectoryOffset, centralDirectorySize, rawEntryCount, comment, commentBytes) {
  let readEntryCursor = 0;
  const allEntriesBuffer = await readAs(reader, centralDirectoryOffset, centralDirectorySize);
  const rawEntries = [];

  for (let e = 0; e < rawEntryCount; ++e) {
    const buffer = allEntriesBuffer.subarray(readEntryCursor, readEntryCursor + 46);
    // 0 - Central directory file header signature
    const signature = getUint32LE(buffer, 0);
    if (signature !== CENTRAL_DIRECTORY_FILE_HEADER_SIGNATURE) {
      throw new Error(`invalid central directory file header signature: 0x${signature.toString(16)}`);
    }
    const rawEntry = {
      // 4 - Version made by
      versionMadeBy: getUint16LE(buffer, 4),
      // 6 - Version needed to extract (minimum)
      versionNeededToExtract: getUint16LE(buffer, 6),
      // 8 - General purpose bit flag
      generalPurposeBitFlag: getUint16LE(buffer, 8),
      // 10 - Compression method
      compressionMethod: getUint16LE(buffer, 10),
      // 12 - File last modification time
      lastModFileTime: getUint16LE(buffer, 12),
      // 14 - File last modification date
      lastModFileDate: getUint16LE(buffer, 14),
      // 16 - CRC-32
      crc32: getUint32LE(buffer, 16),
      // 20 - Compressed size
      compressedSize: getUint32LE(buffer, 20),
      // 24 - Uncompressed size
      uncompressedSize: getUint32LE(buffer, 24),
      // 28 - File name length (n)
      fileNameLength: getUint16LE(buffer, 28),
      // 30 - Extra field length (m)
      extraFieldLength: getUint16LE(buffer, 30),
      // 32 - File comment length (k)
      fileCommentLength: getUint16LE(buffer, 32),
      // 34 - Disk number where file starts
      // 36 - Internal file attributes
      internalFileAttributes: getUint16LE(buffer, 36),
      // 38 - External file attributes
      externalFileAttributes: getUint32LE(buffer, 38),
      // 42 - Relative offset of local file header
      relativeOffsetOfLocalHeader: getUint32LE(buffer, 42),
    };

    if (rawEntry.generalPurposeBitFlag & 0x40) {
      throw new Error('strong encryption is not supported');
    }

    readEntryCursor += 46;

    const data = allEntriesBuffer.subarray(readEntryCursor, readEntryCursor + rawEntry.fileNameLength + rawEntry.extraFieldLength + rawEntry.fileCommentLength);
    rawEntry.nameBytes = data.slice(0, rawEntry.fileNameLength);
    rawEntry.name = decodeBuffer(rawEntry.nameBytes);

    // 46+n - Extra field
    const fileCommentStart = rawEntry.fileNameLength + rawEntry.extraFieldLength;
    const extraFieldBuffer = data.slice(rawEntry.fileNameLength, fileCommentStart);
    rawEntry.extraFields = [];
    let i = 0;
    while (i < extraFieldBuffer.length - 3) {
      const headerId = getUint16LE(extraFieldBuffer, i + 0);
      const dataSize = getUint16LE(extraFieldBuffer, i + 2);
      const dataStart = i + 4;
      const dataEnd = dataStart + dataSize;
      if (dataEnd > extraFieldBuffer.length) {
        throw new Error('extra field length exceeds extra field buffer size');
      }
      rawEntry.extraFields.push({
        id: headerId,
        data: extraFieldBuffer.slice(dataStart, dataEnd),
      });
      i = dataEnd;
    }

    // 46+n+m - File comment
    rawEntry.commentBytes = data.slice(fileCommentStart, fileCommentStart + rawEntry.fileCommentLength);
    rawEntry.comment = decodeBuffer(rawEntry.commentBytes);

    readEntryCursor += data.length;

    if (rawEntry.uncompressedSize            === 0xffffffff ||
        rawEntry.compressedSize              === 0xffffffff ||
        rawEntry.relativeOffsetOfLocalHeader === 0xffffffff) {
      // ZIP64 format
      // find the Zip64 Extended Information Extra Field
      const zip64ExtraField = rawEntry.extraFields.find(e => e.id === 0x0001);
      if (!zip64ExtraField) {
        throw new Error('expected zip64 extended information extra field');
      }
      const zip64EiefBuffer = zip64ExtraField.data;
      let index = 0;
      // 0 - Original Size          8 bytes
      if (rawEntry.uncompressedSize === 0xffffffff) {
        if (index + 8 > zip64EiefBuffer.length) {
          throw new Error('zip64 extended information extra field does not include uncompressed size');
        }
        rawEntry.uncompressedSize = getUint64LE(zip64EiefBuffer, index);
        index += 8;
      }
      // 8 - Compressed Size        8 bytes
      if (rawEntry.compressedSize === 0xffffffff) {
        if (index + 8 > zip64EiefBuffer.length) {
          throw new Error('zip64 extended information extra field does not include compressed size');
        }
        rawEntry.compressedSize = getUint64LE(zip64EiefBuffer, index);
        index += 8;
      }
      // 16 - Relative Header Offset 8 bytes
      if (rawEntry.relativeOffsetOfLocalHeader === 0xffffffff) {
        if (index + 8 > zip64EiefBuffer.length) {
          throw new Error('zip64 extended information extra field does not include relative header offset');
        }
        rawEntry.relativeOffsetOfLocalHeader = getUint64LE(zip64EiefBuffer, index);
        index += 8;
      }
      // 24 - Disk Start Number      4 bytes
    }

    // check for Info-ZIP Unicode Path Extra Field (0x7075)
    // see https://github.com/thejoshwolfe/yauzl/issues/33
    const nameField = rawEntry.extraFields.find(e =>
        e.id === 0x7075 &&
        e.data.length >= 6 && // too short to be meaningful
        e.data[0] === 1 &&    // Version       1 byte      version of this extra field, currently 1
        getUint32LE(e.data, 1), crc$1.unsigned(rawEntry.nameBytes)); // NameCRC32     4 bytes     File Name Field CRC32 Checksum
                                                                   // > If the CRC check fails, this UTF-8 Path Extra Field should be
                                                                   // > ignored and the File Name field in the header should be used instead.
    if (nameField) {
        // UnicodeName Variable UTF-8 version of the entry File Name
        rawEntry.fileName = decodeBuffer(nameField.data.slice(5));
    }

    // validate file size
    if (rawEntry.compressionMethod === 0) {
      let expectedCompressedSize = rawEntry.uncompressedSize;
      if ((rawEntry.generalPurposeBitFlag & 0x1) !== 0) {
        // traditional encryption prefixes the file data with a header
        expectedCompressedSize += 12;
      }
      if (rawEntry.compressedSize !== expectedCompressedSize) {
        throw new Error(`compressed size mismatch for stored file: ${rawEntry.compressedSize} != ${expectedCompressedSize}`);
      }
    }
    rawEntries.push(rawEntry);
  }
  const zip = {
    comment,
    commentBytes,
  };
  return {
    zip,
    entries: rawEntries.map(e => new ZipEntry(reader, e)),
  };
}

async function readEntryDataHeader(reader, rawEntry) {
  if (rawEntry.generalPurposeBitFlag & 0x1) {
    throw new Error('encrypted entries not supported');
  }
  const buffer = await readAs(reader, rawEntry.relativeOffsetOfLocalHeader, 30);
  // note: maybe this should be passed in or cached on entry
  // as it's async so there will be at least one tick (not sure about that)
  const totalLength = await reader.getLength();

  // 0 - Local file header signature = 0x04034b50
  const signature = getUint32LE(buffer, 0);
  if (signature !== 0x04034b50) {
    throw new Error(`invalid local file header signature: 0x${signature.toString(16)}`);
  }

  // all this should be redundant
  // 4 - Version needed to extract (minimum)
  // 6 - General purpose bit flag
  // 8 - Compression method
  // 10 - File last modification time
  // 12 - File last modification date
  // 14 - CRC-32
  // 18 - Compressed size
  // 22 - Uncompressed size
  // 26 - File name length (n)
  const fileNameLength = getUint16LE(buffer, 26);
  // 28 - Extra field length (m)
  const extraFieldLength = getUint16LE(buffer, 28);
  // 30 - File name
  // 30+n - Extra field
  const localFileHeaderEnd = rawEntry.relativeOffsetOfLocalHeader + buffer.length + fileNameLength + extraFieldLength;
  let decompress;
  if (rawEntry.compressionMethod === 0) {
    // 0 - The file is stored (no compression)
    decompress = false;
  } else if (rawEntry.compressionMethod === 8) {
    // 8 - The file is Deflated
    decompress = true;
  } else {
    throw new Error(`unsupported compression method: ${rawEntry.compressionMethod}`);
  }
  const fileDataStart = localFileHeaderEnd;
  const fileDataEnd = fileDataStart + rawEntry.compressedSize;
  if (rawEntry.compressedSize !== 0) {
    // bounds check now, because the read streams will probably not complain loud enough.
    // since we're dealing with an unsigned offset plus an unsigned size,
    // we only have 1 thing to check for.
    if (fileDataEnd > totalLength) {
      throw new Error(`file data overflows file bounds: ${fileDataStart} +  ${rawEntry.compressedSize}  > ${totalLength}`);
    }
  }
  return {
    decompress,
    fileDataStart,
  };
}

async function readEntryDataAsArrayBuffer(reader, rawEntry) {
  const {decompress, fileDataStart} = await readEntryDataHeader(reader, rawEntry);
  if (!decompress) {
    const dataView = await readAs(reader, fileDataStart, rawEntry.compressedSize);
    // make copy?
    //
    // 1. The source is a Blob/file. In this case we'll get back TypedArray we can just hand to the user
    // 2. The source is a TypedArray. In this case we'll get back TypedArray that is a view into a larger buffer
    //    but because ultimately this is used to return an ArrayBuffer to `someEntry.arrayBuffer()`
    //    we need to return copy since we need the `ArrayBuffer`, not the TypedArray to exactly match the data.
    //    Note: We could add another API function `bytes()` or something that returned a `Uint8Array`
    //    instead of an `ArrayBuffer`. This would let us skip a copy here. But this case only happens for uncompressed
    //    data. That seems like a rare enough case that adding a new API is not worth it? Or is it? A zip of jpegs or mp3s
    //    might not be compressed. For now that's a TBD.
    return isTypedArraySameAsArrayBuffer(dataView) ? dataView.buffer : dataView.slice().buffer;
  }
  // see comment in readEntryDateAsBlob
  const typedArrayOrBlob = await readAsBlobOrTypedArray(reader, fileDataStart, rawEntry.compressedSize);
  const result = await inflateRawAsync(typedArrayOrBlob, rawEntry.uncompressedSize);
  return result;
}

async function readEntryDataAsBlob(reader, rawEntry, type) {
  const {decompress, fileDataStart} = await readEntryDataHeader(reader, rawEntry);
  if (!decompress) {
    const typedArrayOrBlob = await readAsBlobOrTypedArray(reader, fileDataStart, rawEntry.compressedSize, type);
    if (isBlob(typedArrayOrBlob)) {
      return typedArrayOrBlob;
    }
    return new Blob([isSharedArrayBuffer(typedArrayOrBlob.buffer) ? new Uint8Array(typedArrayOrBlob) : typedArrayOrBlob], {type});
  }
  // Here's the issue with this mess (should refactor?)
  // if the source is a blob then we really want to pass a blob to inflateRawAsync to avoid a large
  // copy if we're going to a worker.
  const typedArrayOrBlob = await readAsBlobOrTypedArray(reader, fileDataStart, rawEntry.compressedSize);
  const result = await inflateRawAsync(typedArrayOrBlob, rawEntry.uncompressedSize, type);
  return result;
}

function setOptions$1(options) {
  setOptions(options);
}

async function unzipRaw(source) {
  let reader;
  if (typeof Blob !== 'undefined' && source instanceof Blob) {
    reader = new BlobReader(source);
  } else if (source instanceof ArrayBuffer || (source && source.buffer && source.buffer instanceof ArrayBuffer)) {
    reader = new ArrayBufferReader(source);
  } else if (isSharedArrayBuffer(source) || isSharedArrayBuffer(source.buffer)) {
    reader = new ArrayBufferReader(source);
  } else if (typeof source === 'string') {
    const req = await fetch(source);
    if (!req.ok) {
      throw new Error(`failed http request ${source}, status: ${req.status}: ${req.statusText}`);
    }
    const blob = await req.blob();
    reader = new BlobReader(blob);
  } else if (typeof source.getLength === 'function' && typeof source.read === 'function') {
    reader = source;
  } else {
    throw new Error('unsupported source type');
  }

  const totalLength = await reader.getLength();

  if (totalLength > Number.MAX_SAFE_INTEGER) {
    throw new Error(`file too large. size: ${totalLength}. Only file sizes up 4503599627370496 bytes are supported`);
  }

  return await findEndOfCentralDirector(reader, totalLength);
}

// If the names are not utf8 you should use unzipitRaw
async function unzip(source) {
  const {zip, entries} = await unzipRaw(source);
  return {
    zip,
    entries: Object.fromEntries(entries.map(v => [v.name, v])),
  };
}

function cleanup$1() {
  cleanup();
}




/***/ }),

/***/ "assert":
/*!*************************!*\
  !*** external "assert" ***!
  \*************************/
/***/ ((module) => {

"use strict";
module.exports = require("assert");

/***/ }),

/***/ "buffer":
/*!*************************!*\
  !*** external "buffer" ***!
  \*************************/
/***/ ((module) => {

"use strict";
module.exports = require("buffer");

/***/ }),

/***/ "events":
/*!*************************!*\
  !*** external "events" ***!
  \*************************/
/***/ ((module) => {

"use strict";
module.exports = require("events");

/***/ }),

/***/ "fs":
/*!*********************!*\
  !*** external "fs" ***!
  \*********************/
/***/ ((module) => {

"use strict";
module.exports = require("fs");

/***/ }),

/***/ "path":
/*!***********************!*\
  !*** external "path" ***!
  \***********************/
/***/ ((module) => {

"use strict";
module.exports = require("path");

/***/ }),

/***/ "process":
/*!**************************!*\
  !*** external "process" ***!
  \**************************/
/***/ ((module) => {

"use strict";
module.exports = require("process");

/***/ }),

/***/ "stream":
/*!*************************!*\
  !*** external "stream" ***!
  \*************************/
/***/ ((module) => {

"use strict";
module.exports = require("stream");

/***/ }),

/***/ "util":
/*!***********************!*\
  !*** external "util" ***!
  \***********************/
/***/ ((module) => {

"use strict";
module.exports = require("util");

/***/ }),

/***/ "../../node_modules/@ltd/j-toml/index.mjs":
/*!************************************************!*\
  !*** ../../node_modules/@ltd/j-toml/index.mjs ***!
  \************************************************/
/***/ ((__unused_webpack___webpack_module__, __webpack_exports__, __webpack_require__) => {

"use strict";
__webpack_require__.r(__webpack_exports__);
/* harmony export */ __webpack_require__.d(__webpack_exports__, {
/* harmony export */   Keys: () => (/* binding */ Keys),
/* harmony export */   LocalDate: () => (/* binding */ LocalDate),
/* harmony export */   LocalDateTime: () => (/* binding */ LocalDateTime),
/* harmony export */   LocalTime: () => (/* binding */ LocalTime),
/* harmony export */   OffsetDateTime: () => (/* binding */ OffsetDateTime),
/* harmony export */   Section: () => (/* binding */ Section),
/* harmony export */   basic: () => (/* binding */ basic),
/* harmony export */   commentFor: () => (/* binding */ commentFor),
/* harmony export */   commentForThis: () => (/* binding */ commentForThis),
/* harmony export */   "default": () => (/* binding */ _export),
/* harmony export */   inline: () => (/* binding */ inline),
/* harmony export */   isInline: () => (/* binding */ isInline),
/* harmony export */   isSection: () => (/* binding */ isSection),
/* harmony export */   literal: () => (/* binding */ literal),
/* harmony export */   multiline: () => (/* binding */ multiline),
/* harmony export */   parse: () => (/* binding */ parse$1),
/* harmony export */   stringify: () => (/* binding */ stringify),
/* harmony export */   version: () => (/* binding */ version)
/* harmony export */ });
﻿const version = '1.38.0';

const SyntaxError$1 = SyntaxError;

const RangeError$1 = RangeError;

const TypeError$1 = TypeError;

const Error$1 = {if:Error}.if;

const undefined$1 = void null;

const BigInt$1 = typeof BigInt==='undefined' ? undefined$1 : BigInt;

const RegExp$1 = RegExp;

const WeakMap$1 = WeakMap;

const get = WeakMap.prototype.get;

const set = WeakMap.prototype.set;

const create$1 = Object.create;

const isSafeInteger = Number.isSafeInteger;

const getOwnPropertyNames = Object.getOwnPropertyNames;

const freeze = Object.freeze;

const isPrototypeOf = Object.prototype.isPrototypeOf;

const NULL = (
	/* j-globals: null.prototype (internal) */
	Object.seal
		? /*#__PURE__*/Object.preventExtensions(Object.create(null))
		: null
	/* j-globals: null.prototype (internal) */
);

const bind = Function.prototype.bind;

const test = RegExp.prototype.test;

const exec = RegExp.prototype.exec;

const apply$1 = Reflect.apply;

const Proxy$1 = Proxy;

const toStringTag = typeof Symbol==='undefined' ? undefined$1 : Symbol.toStringTag;

const Object_defineProperty = Object.defineProperty;

const assign$1 = Object.assign;

const Object$1 = Object;

const floor = Math.floor;

const isArray$1 = Array.isArray;

const Infinity = 1/0;

const fromCharCode = String.fromCharCode;

const Array$1 = Array;

const hasOwnProperty = Object.prototype.hasOwnProperty;

const propertyIsEnumerable = Object.prototype.propertyIsEnumerable;

const apply = Function.prototype.apply;

var isEnum = /*#__PURE__*/propertyIsEnumerable.call.bind(propertyIsEnumerable);
var hasOwn = (
	/* j-globals: Object.hasOwn (polyfill) */
	Object$1.hasOwn || /*#__PURE__*/function () {
		return hasOwnProperty.bind
			? hasOwnProperty.call.bind(hasOwnProperty)
			: function hasOwn (object, key) { return hasOwnProperty.call(object, key); };
	}()
	/* j-globals: Object.hasOwn (polyfill) */
);

var create = Object$1.create;
function Descriptor (source) {
	var target = create(NULL);
	if ( hasOwn(source, 'value') ) { target.value = source.value; }
	if ( hasOwn(source, 'writable') ) { target.writable = source.writable; }
	if ( hasOwn(source, 'get') ) { target.get = source.get; }
	if ( hasOwn(source, 'set') ) { target.set = source.set; }
	if ( hasOwn(source, 'enumerable') ) { target.enumerable = source.enumerable; }
	if ( hasOwn(source, 'configurable') ) { target.configurable = source.configurable; }
	return target;
}

const Default = (
	/* j-globals: default (internal) */
	function Default (exports, addOnOrigin) {
		if ( !addOnOrigin && typeof exports!=='function' ) {
			addOnOrigin = exports;
			exports = create$1(NULL);
		}
		if ( assign$1 ) { assign$1(exports, addOnOrigin); }
		else { for ( var key in addOnOrigin ) { if ( hasOwn(addOnOrigin, key) ) { exports[key] = addOnOrigin[key]; } } }
		exports.default = exports;
		if ( typeof exports==='function' ) { exports.prototype && freeze(exports.prototype); }
		else if ( toStringTag ) {
			var descriptor = create$1(NULL);
			descriptor.value = 'Module';
			Object_defineProperty(exports, toStringTag, descriptor);
		}
		return freeze(exports);
	}
	/* j-globals: default (internal) */
);

/*!@preserve@license
 * 模块名称：j-regexp
 * 模块功能：可读性更好的正则表达式创建方式。从属于“简计划”。
   　　　　　More readable way for creating RegExp. Belong to "Plan J".
 * 模块版本：8.2.0
 * 许可条款：LGPL-3.0
 * 所属作者：龙腾道 <LongTengDao@LongTengDao.com> (www.LongTengDao.com)
 * 问题反馈：https://GitHub.com/LongTengDao/j-regexp/issues
 * 项目主页：https://GitHub.com/LongTengDao/j-regexp/
 */

var Test                                           = bind
	? /*#__PURE__*/bind.bind(test       )       
	: function (re) {
		return function (string) {
			return test.call(re, string);
		};
	};

var Exec                                           = bind
	? /*#__PURE__*/bind.bind(exec       )       
	: function (re) {
		return function (string) {
			return exec.call(re, string);
		};
	};

function __PURE__ (re        )         {
	var test = re.test = Test(re);
	var exec = re.exec = Exec(re);
	var source = test.source = exec.source = re.source;
	test.unicode = exec.unicode = re.unicode;
	test.ignoreCase = exec.ignoreCase = re.ignoreCase;
	test.multiline = exec.multiline = source.indexOf('^')<0 && source.indexOf('$')<0 ? null : re.multiline;
	test.dotAll = exec.dotAll = source.indexOf('.')<0 ? null : re.dotAll;
	return re;
}
function theRegExp (re        )         { return /*#__PURE__*/__PURE__(re); }

var NT = /[\n\t]+/g;
var ESCAPE = /\\./g;
function graveAccentReplacer ($$        ) { return $$==='\\`' ? '`' : $$; }

var includes = ''.includes       
	? function (that        , searchString        ) { return that.includes(searchString); }
	: function (that        , searchString        ) { return that.indexOf(searchString)>-1; };

function RE (               template                      ) {
	var U = this.U;
	var I = this.I;
	var M = this.M;
	var S = this.S;
	var raw = template.raw;
	var source = raw[0] .replace(NT, '');
	var index = 1;
	var length = arguments.length;
	while ( index!==length ) {
		var value            
			                       
			                          
			                             
			                            
			                         
		  = arguments[index];
		if ( typeof value==='string' ) { source += value; }
		else {
			var value_source = value.source;
			if ( typeof value_source!=='string' ) { throw TypeError$1('source'); }
			if ( value.unicode===U ) { throw SyntaxError$1('unicode'); }
			if ( value.ignoreCase===I ) { throw SyntaxError$1('ignoreCase'); }
			if ( value.multiline===M && ( includes(value_source, '^') || includes(value_source, '$') ) ) { throw SyntaxError$1('multiline'); }
			if ( value.dotAll===S && includes(value_source, '.') ) { throw SyntaxError$1('dotAll'); }
			source += value_source;
		}
		source += raw[index++] .replace(NT, '');
	}
	var re         = RegExp$1(U ? source = source.replace(ESCAPE, graveAccentReplacer) : source, this.flags);
	var test = re.test = Test(re);
	var exec = re.exec = Exec(re);
	test.source = exec.source = source;
	test.unicode = exec.unicode = !U;
	test.ignoreCase = exec.ignoreCase = !I;
	test.multiline = exec.multiline = includes(source, '^') || includes(source, '$') ? !M : null;
	test.dotAll = exec.dotAll = includes(source, '.') ? !S : null;
	return re;
}

var RE_bind = bind && /*#__PURE__*/bind.bind(RE       );

function Context (flags        )          {
	return {
		U: !includes(flags, 'u'),
		I: !includes(flags, 'i'),
		M: !includes(flags, 'm'),
		S: !includes(flags, 's'),
		flags: flags
	};
}

var CONTEXT          = /*#__PURE__*/Context('');

var newRegExp = Proxy$1
	? /*#__PURE__*/new Proxy$1(RE, {
		apply: function (RE, thisArg, args                                   ) { return apply$1(RE, CONTEXT, args); }
		,
		get: function (RE, flags        ) { return RE_bind(Context(flags)); }
		,
		defineProperty: function () { return false; }
		,
		preventExtensions: function () { return false; }
	})
	: /*#__PURE__*/function () {
		RE.apply = RE.apply;
		var newRegExp = function () { return RE.apply(CONTEXT, arguments       ); }       ;
		var d = 1;
		var g = d*2;
		var i = g*2;
		var m = i*2;
		var s = i*2;
		var u = s*2;
		var y = u*2;
		var flags = y*2 - 1;
		while ( flags-- ) {
			( function (context) {
				newRegExp[context.flags] = function () { return RE.apply(context, arguments       ); };
			} )(Context(
				( flags & d ? '' : 'd' )
				+
				( flags & g ? '' : 'g' )
				+
				( flags & i ? '' : 'i' )
				+
				( flags & m ? '' : 'm' )
				+
				( flags & s ? '' : 's' )
				+
				( flags & u ? '' : 'u' )
				+
				( flags & y ? '' : 'y' )
			));
		}
		return freeze ? freeze(newRegExp) : newRegExp;
	}();

var clearRegExp = '$_' in RegExp$1
	? /*#__PURE__*/function () {
		var REGEXP = /^/;
		REGEXP.test = REGEXP.test;
		return function clearRegExp                (value    )                {
			REGEXP.test('');
			return value;
		};
	}()
	: function clearRegExp                (value    )                {
		return value;
	};

var clearRegExp$1 = clearRegExp;

var NEED_TO_ESCAPE_IN_REGEXP = /^[$()*+\-.?[\\\]^{|]/;
var SURROGATE_PAIR = /^[\uD800-\uDBFF][\uDC00-\uDFFF]/;
var GROUP = /*#__PURE__*/create$1(NULL)         ;

function groupify (branches                   , uFlag          , noEscape          )         {
	var group = create$1(NULL)         ;
	var appendBranch = uFlag ? appendPointBranch : appendCodeBranch;
	for ( var length         = branches.length, index         = 0; index<length; ++index ) { appendBranch(group, branches[index] ); }
	return sourcify(group, !noEscape);
}
function appendPointBranch (group       , branch        )       {
	if ( branch ) {
		var character         = SURROGATE_PAIR.test(branch) ? branch.slice(0, 2) : branch.charAt(0);
		appendPointBranch(group[character] || ( group[character] = create$1(NULL)          ), branch.slice(character.length));
	}
	else { group[''] = GROUP; }
}

function appendCodeBranch (group       , branch        )       {
	if ( branch ) {
		var character         = branch.charAt(0);
		appendCodeBranch(group[character] || ( group[character] = create$1(NULL)          ), branch.slice(1));
	}
	else { group[''] = GROUP; }
}

function sourcify (group       , needEscape         )         {
	var branches           = [];
	var singleCharactersBranch           = [];
	var noEmptyBranch          = true;
	for ( var character in group ) {
		if ( character ) {
			var sub_branches         = sourcify(group[character] , needEscape);
			if ( needEscape && NEED_TO_ESCAPE_IN_REGEXP.test(character) ) { character = '\\' + character; }
			sub_branches ? branches.push(character + sub_branches) : singleCharactersBranch.push(character);
		}
		else { noEmptyBranch = false; }
	}
	singleCharactersBranch.length && branches.unshift(singleCharactersBranch.length===1 ? singleCharactersBranch[0]  : '[' + singleCharactersBranch.join('') + ']');
	return branches.length===0
		? ''
		: ( branches.length===1 && ( singleCharactersBranch.length || noEmptyBranch )
			? branches[0]
			: '(?:' + branches.join('|') + ')'
		)
		+ ( noEmptyBranch ? '' : '?' );
}

/*¡ j-regexp */

const WeakSet$1 = WeakSet;

const has = WeakSet.prototype.has;

const add = WeakSet.prototype.add;

const del = WeakSet.prototype['delete'];

const keys = Object.keys;

const getOwnPropertySymbols = Object.getOwnPropertySymbols;

const Null$1 = (
	/* j-globals: null (internal) */
	/*#__PURE__*/function () {
		var assign = Object.assign || function assign (target, source) {
			var keys$1, index, key;
			for ( keys$1 = keys(source), index = 0; index<keys$1.length;++index ) {
				key = keys$1[index];
				target[key] = source[key];
			}
			if ( getOwnPropertySymbols ) {
				for ( keys$1 = getOwnPropertySymbols(source), index = 0; index<keys$1.length;++index ) {
					key = keys$1[index];
					if ( isEnum(source, key) ) { target[key] = source[key]; }
				}
			}
			return target;
		};
		function Nullify (constructor) {
			delete constructor.prototype.constructor;
			freeze(constructor.prototype);
			return constructor;
		}
		function Null (origin) {
			return origin===undefined$1
				? this
				: typeof origin==='function'
					? /*#__PURE__*/Nullify(origin)
					: /*#__PURE__*/assign(/*#__PURE__*/create(NULL), origin);
		}
		delete Null.name;
		//try { delete Null.length; } catch (error) {}
		Null.prototype = null;
		freeze(Null);
		return Null;
	}()
	/* j-globals: null (internal) */
);

const is = Object.is;

const Object_defineProperties = Object.defineProperties;

const fromEntries = Object.fromEntries;

const Reflect_construct = Reflect.construct;

const Reflect_defineProperty = Reflect.defineProperty;

const Reflect_deleteProperty = Reflect.deleteProperty;

const ownKeys = Reflect.ownKeys;

/*!@preserve@license
 * 模块名称：j-orderify
 * 模块功能：返回一个能保证给定对象的属性按此后添加顺序排列的 proxy，即使键名是 symbol，或整数 string。从属于“简计划”。
   　　　　　Return a proxy for given object, which can guarantee own keys are in setting order, even if the key name is symbol or int string. Belong to "Plan J".
 * 模块版本：7.0.1
 * 许可条款：LGPL-3.0
 * 所属作者：龙腾道 <LongTengDao@LongTengDao.com> (www.LongTengDao.com)
 * 问题反馈：https://GitHub.com/LongTengDao/j-orderify/issues
 * 项目主页：https://GitHub.com/LongTengDao/j-orderify/
 */

const Keeper =     ()      => [];

const newWeakMap = () => {
	const weakMap = new WeakMap$1;
	weakMap.has = weakMap.has;
	weakMap.get = weakMap.get;
	weakMap.set = weakMap.set;
	return weakMap;
};
const target2keeper = /*#__PURE__*/newWeakMap()     
	                                                                      
	                                                                         
 ;
const proxy2target = /*#__PURE__*/newWeakMap()     
	                             
	                                                 
	                                                   
 ;
const target2proxy = /*#__PURE__*/newWeakMap()     
	                                                  
	                                                   
 ;

const handlers                       = /*#__PURE__*/assign$1(create$1(NULL), {
	defineProperty:                 (target                   , key   , descriptor                    )          => {
		if ( hasOwn(target, key) ) {
			return Reflect_defineProperty(target, key, assign$1(create$1(NULL), descriptor));
		}
		if ( Reflect_defineProperty(target, key, assign$1(create$1(NULL), descriptor)) ) {
			const keeper = target2keeper.get(target) ;
			keeper[keeper.length] = key;
			return true;
		}
		return false;
	},
	deleteProperty:                 (target                   , key   )          => {
		if ( Reflect_deleteProperty(target, key) ) {
			const keeper = target2keeper.get(target) ;
			const index = keeper.indexOf(key);
			index<0 || --keeper.copyWithin(index, index + 1).length;
			return true;
		}
		return false;
	},
	ownKeys:                    (target   ) => target2keeper.get(target)                         ,
	construct:                                     (target                         , args   , newTarget     )    => orderify(Reflect_construct(target, args, newTarget)),
	apply:                                        (target                              , thisArg   , args   )    => orderify(apply$1(target, thisArg, args)),
});

const newProxy =                                              (target   , keeper           )    => {
	target2keeper.set(target, keeper);
	const proxy = new Proxy$1   (target, handlers);
	proxy2target.set(proxy, target);
	return proxy;
};

const orderify =                    (object   )    => {
	if ( proxy2target.has(object) ) { return object; }
	let proxy = target2proxy.get(object)                 ;
	if ( proxy ) { return proxy; }
	proxy = newProxy(object, assign$1(Keeper          (), ownKeys(object)));
	target2proxy.set(object, proxy);
	return proxy;
};

const Null = /*#__PURE__*/function () {
	function throwConstructing ()        { throw TypeError$1(`Super constructor Null cannot be invoked with 'new'`); }
	function throwApplying ()        { throw TypeError$1(`Super constructor Null cannot be invoked without 'new'`); }
	const Nullify = (constructor                             ) => {
		delete constructor.prototype.constructor;
		freeze(constructor.prototype);
		return constructor;
	};
	function Null (           constructor                              ) {
		return new.target
			? new.target===Null
				? /*#__PURE__*/throwConstructing()
				: /*#__PURE__*/newProxy(this, Keeper     ())
			: typeof constructor==='function'
				? /*#__PURE__*/Nullify(constructor)
				: /*#__PURE__*/throwApplying();
	}
	//@ts-ignore
	Null.prototype = null;
	Object_defineProperty(Null, 'name', assign$1(create$1(NULL), { value: '', configurable: false }));
	//delete Null.length;
	freeze(Null);
	return Null;
}()                                           ;

/*¡ j-orderify */

const map_has = WeakMap.prototype.has;

const map_del = WeakMap.prototype['delete'];

const INLINES = new WeakMap$1                                                                     ();
const SECTIONS = new WeakSet$1                ();

const deInline = /*#__PURE__*/map_del.bind(INLINES)                                                                              ;
const deSection = /*#__PURE__*/del.bind(SECTIONS)                                                  ;

const isInline = /*#__PURE__*/map_has.bind(INLINES)                                                  ;
const ofInline = /*#__PURE__*/get.bind(INLINES)     
	                                                                          
	                                                               
	                                       
 ;
const beInline = /*#__PURE__*/set.bind(INLINES)     
	                                                                                  
	                                                                       
 ;
const inline =                                                         (value   , mode                , looping         )    => {
	if ( isArray$1(value) ) {
		if ( looping ) { mode = 3; }
		else {
			if ( mode===undefined$1 ) { mode = 3; }
			else if ( mode!==0 && mode!==1 && mode!==2 && mode!==3 ) {
				throw typeof mode==='number'
					? RangeError$1(`array inline mode must be 0 | 1 | 2 | 3, not including ${mode}`)
					: TypeError$1(`array inline mode must be "number" type, not including ${mode===null ? '"null"' : typeof mode}`);
			}
		}
		beInline(value, mode);
	}
	else {
		beInline(value, true);
		deSection(value);
	}
	return value;
};
const multilineTable =                                  (value   )    => {
	beInline(value, false);
	deSection(value);
	return value;
};
const multilineArray =                                       (value   )    => {
	deInline(value);
	return value;
};

const isSection = /*#__PURE__*/has.bind(SECTIONS)                                                                  ;
const beSection = /*#__PURE__*/add.bind(SECTIONS)                                                 ;
const Section =                            (table   )    => {
	if ( isArray$1(table) ) { throw TypeError$1(`array can not be section, maybe you want to use it on the tables in it`); }
	beSection(table);
	deInline(table);
	return table;
};

const INLINE = true;

const tables = new WeakSet$1       ();
const tables_add = /*#__PURE__*/add.bind(tables);
const isTable = /*#__PURE__*/has.bind(tables)                                              ;

const implicitTables = new WeakSet$1       ();
const implicitTables_add = /*#__PURE__*/add.bind(implicitTables);
const implicitTables_del = /*#__PURE__*/del.bind(implicitTables)                                         ;
const directlyIfNot = (table       )          => {
	if ( implicitTables_del(table) ) {
		beSection(table);
		return true;
	}
	return false;
};
const DIRECTLY = true;
const IMPLICITLY = false;

const pairs = new WeakSet$1       ();
const pairs_add = /*#__PURE__*/add.bind(pairs);
const fromPair = /*#__PURE__*/has.bind(pairs)                                         ;
const PAIR = true;

const PlainTable = /*#__PURE__*/Null$1(class Table extends Null$1      {
	                                
	constructor (isDirect          , isInline$fromPair          ) {
		super();
		tables_add(this);
		isDirect
			? isInline$fromPair ? beInline(this, true) : beSection(this)
			: ( isInline$fromPair ? pairs_add : implicitTables_add )(this);
		return this;
	}
});

const OrderedTable = /*#__PURE__*/Null$1(class Table extends Null      {
	                                
	constructor (isDirect          , isInline$fromPair          ) {
		super();
		tables_add(this);
		isDirect
			? isInline$fromPair ? beInline(this, true) : beSection(this)
			: ( isInline$fromPair ? pairs_add : implicitTables_add )(this);
		return this;
	}
});

//import * as options from './options';

const NONE                    = [];
let sourcePath         = '';
let sourceLines                    = NONE;
let lastLineIndex         = -1;
let lineIndex         = -1;

const throws = (error       )        => {
	//if ( sourceLines!==NONE ) { done(); options.clear(); }
	throw error;
};

const EOL = /\r?\n/;
const todo = (source        , path        )       => {
	if ( typeof path!=='string' ) { throw TypeError$1(`TOML.parse({ path })`); }
	sourcePath = path;
	sourceLines = source.split(EOL);
	lastLineIndex = sourceLines.length - 1;
	lineIndex = -1;
};

const next = ()         => sourceLines[++lineIndex] ;

const rest = ()          => lineIndex!==lastLineIndex;

class mark {
	                 lineIndex = lineIndex;
	                 type                                                                                           ;
	                 restColumn        ;
	constructor (type                                                                                           , restColumn        ) {
		this.type = type;
		this.restColumn = restColumn;
		return this;
	}
	must (          )         {
		lineIndex===lastLineIndex && throws(SyntaxError$1(`${this.type} is not close until the end of the file` + where(', which started from ', this.lineIndex, sourceLines[this.lineIndex] .length - this.restColumn + 1)));
		return sourceLines[++lineIndex] ;
	}
	nowrap (            argsMode                 )        {
		throw throws(Error$1(`TOML.parse(${argsMode ? `${argsMode}multilineStringJoiner` : `,{ joiner }`}) must be passed, while the source including multi-line string` + where(', which started from ', this.lineIndex, sourceLines[this.lineIndex] .length - this.restColumn + 1)));
	}
}
const where = (pre        , rowIndex         = lineIndex, columnNumber         = 0)         => sourceLines===NONE ? '' :
	sourcePath
		? `\n    at (${sourcePath}:${rowIndex + 1}:${columnNumber})`
		: `${pre}line ${rowIndex + 1}: ${sourceLines[rowIndex]}`;

const done = ()       => {
	sourcePath = '';
	sourceLines = NONE;
};

/* nested (readable) */

const Whitespace = /[ \t]/;

const PRE_WHITESPACE = /*#__PURE__*/newRegExp`
	^${Whitespace}+`.valueOf();

const { exec: VALUE_REST_exec } = /*#__PURE__*/newRegExp.s       `
	^
	(
		(?:\d\d\d\d-\d\d-\d\d \d)?
		[\w\-+.:]+
	)
	${Whitespace}*
	(.*)
	$`.valueOf();

const { exec: LITERAL_STRING_exec } = /*#__PURE__*/newRegExp.s       `
	^
	'([^']*)'
	${Whitespace}*
	(.*)`.valueOf();

const { exec: MULTI_LINE_LITERAL_STRING_0_1_2 } = /*#__PURE__*/newRegExp.s           `
	^
	(.*?)
	'''('{0,2})
	${Whitespace}*
	(.*)`.valueOf();
const { exec: MULTI_LINE_LITERAL_STRING_0 } = /*#__PURE__*/newRegExp.s           `
	^
	(.*?)
	'''()
	${Whitespace}*
	(.*)`.valueOf();
let __MULTI_LINE_LITERAL_STRING_exec = MULTI_LINE_LITERAL_STRING_0;

const SYM_WHITESPACE = /*#__PURE__*/newRegExp.s`
	^
	.
	${Whitespace}*`.valueOf();


const Tag = /[^\x00-\x1F"#'()<>[\\\]`{}\x7F]+/;

const { exec: KEY_VALUE_PAIR_exec } = /*#__PURE__*/newRegExp.s   `
	^
	${Whitespace}*
	=
	${Whitespace}*
	(?:
		<(${Tag})>
		${Whitespace}*
	)?
	(.*)
	$`.valueOf();

const { exec: _VALUE_PAIR_exec } = /*#__PURE__*/newRegExp.s       `
	^
	<(${Tag})>
	${Whitespace}*
	(.*)
	$`.valueOf();

const { exec: TAG_REST_exec } = /*#__PURE__*/newRegExp.s       `
	^
	<(${Tag})>
	${Whitespace}*
	(.*)
	$`.valueOf();

/* optimized (avoid overflow or lost) */

const MULTI_LINE_BASIC_STRING = theRegExp(/[^\\"]+|\\.?|"(?!"")"?/sy);
const MULTI_LINE_BASIC_STRING_exec_0_length = (_        )         => {
	let lastIndex         = /*MULTI_LINE_BASIC_STRING.lastIndex = */0;
	while ( MULTI_LINE_BASIC_STRING.test(_) ) { lastIndex = MULTI_LINE_BASIC_STRING.lastIndex; }
	return lastIndex;
};

const ESCAPED_EXCLUDE_CONTROL_CHARACTER_TAB______ = /[^\\\x00-\x08\x0B-\x1F\x7F]+|\\(?:[btnfr"\\]|[\t ]*\n[\t\n ]*|u[\dA-Fa-f]{4}|U[\dA-Fa-f]{8})/g;
const ESCAPED_EXCLUDE_CONTROL_CHARACTER__________ = /[^\\\x00-\x09\x0B-\x1F\x7F]+|\\(?:[btnfr"\\]|[\t ]*\n[\t\n ]*|u[\dA-Fa-f]{4}|U[\dA-Fa-f]{8})/g;/// Tab
const ESCAPED_EXCLUDE_CONTROL_CHARACTER_DEL______ = /[^\\\x00-\x09\x0B-\x1F]+|\\(?:[btnfr"\\]|[\t ]*\n[\t\n ]*|u[\dA-Fa-f]{4}|U[\dA-Fa-f]{8})/g;/// Tab \<ws>newline
const ESCAPED_EXCLUDE_CONTROL_CHARACTER_DEL_SLASH = /[^\\\x00-\x09\x0B-\x1F]+|\\(?:[btnfr"\\/]|[\t ]*\n[\t\n ]*|u[\dA-Fa-f]{4}|U[\dA-Fa-f]{8})/g;/// not \<ws>newline
let __ESCAPED_EXCLUDE_CONTROL_CHARACTER = ESCAPED_EXCLUDE_CONTROL_CHARACTER_TAB______;
const ESCAPED_EXCLUDE_CONTROL_CHARACTER_test = (_        )          => !_.replace(__ESCAPED_EXCLUDE_CONTROL_CHARACTER, '');/// op?

const BASIC_STRING_TAB______ = theRegExp(/[^\\"\x00-\x08\x0B-\x1F\x7F]+|\\(?:[btnfr"\\]|u[\dA-Fa-f]{4}|U[\dA-Fa-f]{8})/y);
const BASIC_STRING__________ = theRegExp(/[^\\"\x00-\x08\x0B-\x1F\x7F]+|\\(?:[btnfr"\\]|u[\dA-Fa-f]{4}|U[\dA-Fa-f]{8})/y);/// Tab
const BASIC_STRING_DEL______ = theRegExp(/[^\\"\x00-\x08\x0B-\x1F]+|\\(?:[btnfr"\\]|u[\dA-Fa-f]{4}|U[\dA-Fa-f]{8})/y);/// Tab
const BASIC_STRING_DEL_SLASH = theRegExp(/[^\\"\x00-\x08\x0B-\x1F]+|\\(?:[btnfr"\\/]|u[\dA-Fa-f]{4}|U[\dA-Fa-f]{8})/y);/// Tab
let __BASIC_STRING = BASIC_STRING_DEL_SLASH;
const BASIC_STRING_exec_1_endIndex = (line        )         => {
	let lastIndex         = __BASIC_STRING.lastIndex = 1;
	while ( __BASIC_STRING.test(line) ) { lastIndex = __BASIC_STRING.lastIndex; }
	lastIndex!==line.length && line[lastIndex]==='"' || throws(SyntaxError$1(`Bad basic string` + where(' at ')));
	return lastIndex;
};

const { test: IS_DOT_KEY } = theRegExp(/^[ \t]*\./);
const DOT_KEY = /^[ \t]*\.[ \t]*/;
const { exec: BARE_KEY_STRICT } = theRegExp(/^[\w-]+/);
const { exec: BARE_KEY_FREE } = theRegExp(/^[^ \t#=[\]'".]+(?:[ \t]+[^ \t#=[\]'".]+)*/);
let __BARE_KEY_exec = BARE_KEY_FREE;
const { exec: LITERAL_KEY____ } = theRegExp(/^'[^'\x00-\x08\x0B-\x1F\x7F]*'/);
const { exec: LITERAL_KEY_DEL } = theRegExp(/^'[^'\x00-\x08\x0B-\x1F]*'/);
let __LITERAL_KEY_exec = LITERAL_KEY_DEL;
let supportArrayOfTables = true;

const TABLE_DEFINITION_exec_groups = (lineRest        , parseKeys                                                                                                 )                                                                                                   => {
	const asArrayItem          = lineRest[1]==='[';
	if ( asArrayItem ) {
		supportArrayOfTables || throws(SyntaxError$1(`Array of Tables is not allowed before TOML v0.2` + where(', which at ')));
		lineRest = lineRest.slice(2);
	}
	else { lineRest = lineRest.slice(1); }
	lineRest = lineRest.replace(PRE_WHITESPACE, '');
	const { leadingKeys, finalKey } = { lineRest } = parseKeys(lineRest);
	lineRest = lineRest.replace(PRE_WHITESPACE, '');
	lineRest && lineRest[0]===']' || throws(SyntaxError$1(`Table header is not closed` + where(', which is found at ')));
	( lineRest.length>1 ? lineRest[1]===']'===asArrayItem : !asArrayItem ) || throws(SyntaxError$1(`Square brackets of Table definition statement not match` + where(' at ')));
	lineRest = lineRest.slice(asArrayItem ? 2 : 1).replace(PRE_WHITESPACE, '');
	let tag        ;
	if ( lineRest && lineRest[0]==='<' ) { ( { 1: tag, 2: lineRest } = TAG_REST_exec(lineRest) || throws(SyntaxError$1(`Bad tag` + where(' at '))) ); }
	else { tag = ''; }
	return { leadingKeys, finalKey, asArrayItem, tag, lineRest };
};

const KEY_VALUE_PAIR_exec_groups = ({ leadingKeys, finalKey, lineRest }                                                               )                                                                             => {
	const { 1: tag = '' } = { 2: lineRest } = KEY_VALUE_PAIR_exec(lineRest) || throws(SyntaxError$1(`Keys must equal something` + where(', but missing at ')));
	tag || lineRest && lineRest[0]!=='#' || throws(SyntaxError$1(`Value can not be missing after euqal sign` + where(', which is found at ')));
	return { leadingKeys, finalKey, tag, lineRest };
};

const { test: CONTROL_CHARACTER_EXCLUDE_TAB____ } = theRegExp(/[\x00-\x08\x0B-\x1F\x7F]/);
const { test: CONTROL_CHARACTER_EXCLUDE_TAB_DEL } = theRegExp(/[\x00-\x08\x0B-\x1F]/);
let __CONTROL_CHARACTER_EXCLUDE_test = CONTROL_CHARACTER_EXCLUDE_TAB____;

const switchRegExp = (specificationVersion        )       => {
	switch ( specificationVersion ) {
		case 1.0:
			__MULTI_LINE_LITERAL_STRING_exec = MULTI_LINE_LITERAL_STRING_0_1_2;
			__LITERAL_KEY_exec = LITERAL_KEY____;
			__CONTROL_CHARACTER_EXCLUDE_test = CONTROL_CHARACTER_EXCLUDE_TAB____;
			__ESCAPED_EXCLUDE_CONTROL_CHARACTER = ESCAPED_EXCLUDE_CONTROL_CHARACTER_TAB______;
			__BASIC_STRING = BASIC_STRING_TAB______;
			__BARE_KEY_exec = BARE_KEY_STRICT;
			supportArrayOfTables = true;
			break;
		case 0.5:
			__MULTI_LINE_LITERAL_STRING_exec = MULTI_LINE_LITERAL_STRING_0;
			__LITERAL_KEY_exec = LITERAL_KEY____;
			__CONTROL_CHARACTER_EXCLUDE_test = CONTROL_CHARACTER_EXCLUDE_TAB____;
			__ESCAPED_EXCLUDE_CONTROL_CHARACTER = ESCAPED_EXCLUDE_CONTROL_CHARACTER__________;
			__BASIC_STRING = BASIC_STRING__________;
			__BARE_KEY_exec = BARE_KEY_STRICT;
			supportArrayOfTables = true;
			break;
		case 0.4:
			__MULTI_LINE_LITERAL_STRING_exec = MULTI_LINE_LITERAL_STRING_0;
			__LITERAL_KEY_exec = LITERAL_KEY_DEL;
			__CONTROL_CHARACTER_EXCLUDE_test = CONTROL_CHARACTER_EXCLUDE_TAB_DEL;
			__ESCAPED_EXCLUDE_CONTROL_CHARACTER = ESCAPED_EXCLUDE_CONTROL_CHARACTER_DEL______;
			__BASIC_STRING = BASIC_STRING_DEL______;
			__BARE_KEY_exec = BARE_KEY_STRICT;
			supportArrayOfTables = true;
			break;
		default:
			__MULTI_LINE_LITERAL_STRING_exec = MULTI_LINE_LITERAL_STRING_0;
			__LITERAL_KEY_exec = LITERAL_KEY_DEL;
			__CONTROL_CHARACTER_EXCLUDE_test = CONTROL_CHARACTER_EXCLUDE_TAB_DEL;
			__ESCAPED_EXCLUDE_CONTROL_CHARACTER = ESCAPED_EXCLUDE_CONTROL_CHARACTER_DEL_SLASH;
			__BASIC_STRING = BASIC_STRING_DEL_SLASH;
			__BARE_KEY_exec = BARE_KEY_FREE;
			supportArrayOfTables = false;
	}
};

const NUM = /*#__PURE__*/newRegExp`
	(?:
		0
		(?:
			b[01][_01]*
		|
			o[0-7][_0-7]*
		|
			x[\dA-Fa-f][_\dA-Fa-f]*
		|
			(?:\.\d[_\d]*)?(?:[Ee]-?\d[_\d]*)?
		)
	|
		[1-9][_\d]*
		(?:\.\d[_\d]*)?(?:[Ee]-?\d[_\d]*)?
	|
		inf
	|
		nan
	)
`.valueOf();
const { test: IS_AMAZING } = /*#__PURE__*/newRegExp`
	^(?:
		-?${NUM}
		(?:-${NUM})*
	|
		true
	|
		false
	)$
`.valueOf();
const { test: BAD_DXOB } = /*#__PURE__*/newRegExp`_(?![\dA-Fa-f])`.valueOf();
const isAmazing = (keys        )          => IS_AMAZING(keys) && !BAD_DXOB(keys);

let mustScalar          = true;

let ARGS_MODE                  = '';

/* options */

let useWhatToJoinMultilineString                = null;
let usingBigInt                 = true;
let IntegerMinNumber         = 0;
let IntegerMaxNumber         = 0;

              

                                           
	                 
	                
	                 
	                
	               
	                
	                  
	                 
	                  
	                   
  
const ANY       = {
	test: () => true,
};
                       
	                                                    
 
const Keys = class KeysRegExp extends RegExp$1                 {
	                                   
	constructor (keys                   ) {
		super(`^${groupify(keys)}$`);
		let maxLength = -1;
		for ( let index = keys.length; index; ) {
			const { length } = keys[--index] ;
			if ( length>maxLength ) { maxLength = length; }
		}
		this.lastIndex = maxLength+1;
		return this;
	}
	         test (                  key        )          {
		return key.length<this.lastIndex && super.test(key);
	}
};
const isKeys = /*#__PURE__*/isPrototypeOf.bind(/*#__PURE__*/freeze(Keys.prototype))                                               ;
let KEYS$1       = ANY;
let preserveLiteral         ;
let zeroDatetime         ;
let inlineTable         ;
let moreDatetime         ;
let disallowEmptyKey         ;
//export const xob :boolean = true;
let sError         ;
let sFloat         ;
                               
let Table                  ;
let allowLonger         ;
let enableNull         ;
let allowInlineTableMultilineAndTrailingCommaEvenNoComma         ;
let preserveComment         ;
let disableDigit         ;
const arrayTypes = new WeakMap$1           ();
const arrayTypes_get = /*#__PURE__*/get.bind(arrayTypes)                                  ;
const arrayTypes_set = /*#__PURE__*/set.bind(arrayTypes)                                     ;
                                  
const As = ()     => {
	const as = (array       )        => {
		const got = arrayTypes_get(array);
		got
			? got===as || throws(TypeError$1(`Types in Array must be same` + where('. Check ')))
			: arrayTypes_set(array, as);
		return array;
	};
	return as;
};
const AS_TYPED = {
	asNulls: As(),
	asStrings: As(),
	asTables: As(),
	asArrays: As(),
	asBooleans: As(),
	asFloats: As(),
	asIntegers: As(),
	asOffsetDateTimes: As(),
	asLocalDateTimes: As(),
	asLocalDates: As(),
	asLocalTimes: As(),
};
const asMixed     = (array       )        => array;
let
	asNulls    ,
	asStrings    ,
	asTables    ,
	asArrays    ,
	asBooleans    ,
	asFloats    ,
	asIntegers    ,
	asOffsetDateTimes    ,
	asLocalDateTimes    ,
	asLocalDates    ,
	asLocalTimes    ;

                  

                                            
let processor             = null;
let each              = null;
           
	                                                                                                      
	                                                                                                      
	                                                                                                      
 
const collect_on = (tag        , array              , table              , key         )       => {
	const _each = create$1(NULL)                                                                                                 ;
	_each._linked = each;
	_each.tag = tag;
	if ( table ) {
		_each.table = table;
		_each.key = key ;
	}
	if ( array ) {
		_each.array = array;
		_each.index = array.length;
	}
	each = _each;
};
const collect_off = ()        => { throw throws(SyntaxError$1(`xOptions.tag is not enabled, but found tag syntax` + where(' at '))); };
let collect                                                                                                                          = collect_off;
                                                      
const Process = ()          => {
	if ( each ) {
		const _processor = processor ;
		let _each              = each;
		each = null;
		return ()       => {
			const processor = _processor;
			let each              = _each ;
			_each = null;
			do { processor(each); }
			while ( each = each._linked );
		};
	}
	return null;
};

/* use & clear */

const clear = ()       => {
	KEYS$1 = ANY;
	useWhatToJoinMultilineString = processor = each = null;
	zeroDatetime = false;
};

const use = (specificationVersion         , multilineStringJoiner         , useBigInt         , keys         , xOptions          , argsMode                 )       => {
	
	ARGS_MODE = argsMode;
	
	let mixed         ;
	switch ( specificationVersion ) {
		case 1.0:
			mustScalar = mixed = moreDatetime = sFloat = inlineTable = true;
			zeroDatetime = disallowEmptyKey = false;
			break;
		case 0.5:
			mustScalar = moreDatetime = sFloat = inlineTable = true;
			mixed = zeroDatetime = disallowEmptyKey = false;
			break;
		case 0.4:
			mustScalar = disallowEmptyKey = inlineTable = true;
			mixed = zeroDatetime = moreDatetime = sFloat = false;
			break;
		case 0.3:
			mustScalar = disallowEmptyKey = true;
			mixed = zeroDatetime = moreDatetime = sFloat = inlineTable = false;
			break;
		case 0.2:
			zeroDatetime = disallowEmptyKey = true;
			mustScalar = mixed = moreDatetime = sFloat = inlineTable = false;
			break;
		case 0.1:
			zeroDatetime = disallowEmptyKey = true;
			mustScalar = mixed = moreDatetime = sFloat = inlineTable = false;
			break;
		default:
			throw RangeError$1(`TOML.parse(,specificationVersion)`);
	}
	switchRegExp(specificationVersion);
	
	if ( typeof multilineStringJoiner==='string' ) { useWhatToJoinMultilineString = multilineStringJoiner; }
	else if ( multilineStringJoiner===undefined$1 ) { useWhatToJoinMultilineString = null; }
	else { throw TypeError$1(`TOML.parse(${ARGS_MODE ? `${ARGS_MODE}multilineStringJoiner` : `,{ joiner }`})`); }
	
	if ( useBigInt===undefined$1 || useBigInt===true ) { usingBigInt = true; }
	else if ( useBigInt===false ) { usingBigInt = false; }
	else {
		if ( typeof useBigInt!=='number' ) { throw TypeError$1(`TOML.parse(${ARGS_MODE ? `${ARGS_MODE},useBigInt` : `,{ bigint }`})`); }
		if ( !isSafeInteger(useBigInt) ) { throw RangeError$1(`TOML.parse(${ARGS_MODE ? `${ARGS_MODE},useBigInt` : `,{ bigint }`})`); }
		usingBigInt = null;
		useBigInt>=0
			? IntegerMinNumber = -( IntegerMaxNumber = useBigInt )
			: IntegerMaxNumber = -( IntegerMinNumber = useBigInt ) - 1;
	}
	if ( !BigInt$1 && usingBigInt!==false ) { throw Error$1(`Can't work without TOML.parse(${ARGS_MODE ? `${ARGS_MODE},useBigInt` : `,{ bigint }`}) being set to false, because the host doesn't have BigInt support`); }
	
	if ( keys==null ) { KEYS$1 = ANY; }
	else {
		if ( !isKeys(keys) ) { throw TypeError$1(`TOML.parse(,{ keys })`); }
		KEYS$1 = keys;
	}
	
	if ( xOptions==null ) {
		Table = PlainTable;
		sError = allowLonger = enableNull = allowInlineTableMultilineAndTrailingCommaEvenNoComma = false;
		collect = collect_off;
	}
	else if ( typeof xOptions!=='object' ) {
		throw TypeError$1(`TOML.parse(${ARGS_MODE ? `${ARGS_MODE},,xOptions` : `,{ x }`})`);
	}
	else {
		const { order, longer, exact, null: _null, multi, comment, string, literal, tag, ...unknown } = xOptions;
		const unknownNames = getOwnPropertyNames(unknown);
		if ( unknownNames.length ) { throw TypeError$1(`TOML.parse(${ARGS_MODE ? `${ARGS_MODE},,{ ${unknownNames.join(', ')} }` : `,{ x: { ${unknownNames.join(', ')} } }`})`); }
		Table = order ? OrderedTable : PlainTable;
		allowLonger = !longer;
		sError = !!exact;
		enableNull = !!_null;
		allowInlineTableMultilineAndTrailingCommaEvenNoComma = !!multi;
		preserveComment = !!comment;
		disableDigit = !!string;
		preserveLiteral = !!literal;
		if ( tag ) {
			if ( typeof tag!=='function' ) { throw TypeError$1(`TOML.parse(${ARGS_MODE ? `${ARGS_MODE},,{ tag }` : `,{ x: { tag } }`})`); }
			if ( !mixed ) { throw TypeError$1(`TOML.parse(${ARGS_MODE ? `${ARGS_MODE},,xOptions` : `,{ x }`}) xOptions.tag needs at least TOML 1.0 to support mixed type array`); }
			processor = tag;
			collect = collect_on;
		}
		else { collect = collect_off; }
	}
	
	mixed
		? asNulls = asStrings = asTables = asArrays = asBooleans = asFloats = asIntegers = asOffsetDateTimes = asLocalDateTimes = asLocalDates = asLocalTimes = asMixed
		: ( { asNulls, asStrings, asTables, asArrays, asBooleans, asFloats, asIntegers, asOffsetDateTimes, asLocalDateTimes, asLocalDates, asLocalTimes } = AS_TYPED );
	
};

const isView = ArrayBuffer.isView;

const isArrayBuffer = (
	/* j-globals: class.isArrayBuffer (internal) */
	/*#__PURE__*/ function () {
		if ( typeof ArrayBuffer==='function' ) {
			var byteLength_apply = apply.bind(Object.getOwnPropertyDescriptor(ArrayBuffer.prototype, 'byteLength').get);
			return function isArrayBuffer (value) {
				try { byteLength_apply(value); }
				catch (error) { return false; }
				return true;
			};
		}
		return function isArrayBuffer () { return false; };
	}()
	/* j-globals: class.isArrayBuffer (internal) */
);

const TextDecoder$1 = TextDecoder;

const Symbol$1 = Symbol;

const previous                = Symbol$1('previous')       ;

              
	                                
		                                                  
		                                                  
	                  
  

const x =     (rootStack      )    => {
	let stack        = rootStack;
	let result = stack.next();
	if ( !result.done ) {
		result.value[previous] = stack;
		result = ( stack = result.value ).next();
		for ( ; ; ) {
			if ( result.done ) {
				if ( stack===rootStack ) { break; }
				stack = stack[previous] ;
				result = stack.next(result.value);
			}
			else {
				result.value[previous] = stack;
				result = ( stack = result.value ).next();
			}
		}
	}
	return result.value;
};

const _literal                = Symbol$1('_literal')       ;

const LiteralObject =                                                             (literal         , value                                   ) => {
	const object = Object$1(value)                           ;
	object[_literal] = literal;
	return object;
};

const arrays = new WeakSet$1       ();
const arrays_add = /*#__PURE__*/add.bind(arrays);
const isArray = /*#__PURE__*/has.bind(arrays)                                  ;

const OF_TABLES = false;
const STATICALLY = true;
const staticalArrays = new WeakSet$1       ();
const staticalArrays_add = /*#__PURE__*/add.bind(staticalArrays);
const isStatic = /*#__PURE__*/has.bind(staticalArrays)                             ;

const newArray = (isStatic         )        => {
	const array        = [];
	arrays_add(array);
	isStatic && staticalArrays_add(array);
	return array;
};

const NativeDate = Date;

const parse$2 = Date.parse;

const preventExtensions = Object.preventExtensions;

const getOwnPropertyDescriptors = Object.getOwnPropertyDescriptors;

const defineProperties = (
	/* j-globals: null.defineProperties (internal) */
	function defineProperties (object, descriptorMap) {
		var created = create$1(NULL);
		var names = keys(descriptorMap);
		for ( var length = names.length, index = 0; index<length; ++index ) {
			var name = names[index];
			created[name] = Descriptor(descriptorMap[name]);
		}
		if ( getOwnPropertySymbols ) {
			var symbols = getOwnPropertySymbols(descriptorMap);
			for ( length = symbols.length, index = 0; index<length; ++index ) {
				var symbol = symbols[index];
				if ( isEnum(descriptorMap, symbol) ) { created[symbol] = Descriptor(descriptorMap[symbol]); }
			}
		}
		return Object_defineProperties(object, created);
	}
	/* j-globals: null.defineProperties (internal) */
);

const fpc =                      (c   )    => {
	freeze(freeze(c).prototype);
	return c;
};

const _29_ = /(?:0[1-9]|1\d|2\d)/;
const _30_ = /(?:0[1-9]|[12]\d|30)/;
const _31_ = /(?:0[1-9]|[12]\d|3[01])/;
const _23_ = /(?:[01]\d|2[0-3])/;
const _59_ = /[0-5]\d/;

const YMD = /*#__PURE__*/newRegExp`
	\d\d\d\d-
	(?:
		0
		(?:
			[13578]-${_31_}
			|
			[469]-${_30_}
			|
			2-${_29_}
		)
		|
		1
		(?:
			[02]-${_31_}
			|
			1-${_30_}
		)
	)
`.valueOf();

const HMS = /*#__PURE__*/newRegExp`
	${_23_}:${_59_}:${_59_}
`.valueOf();

const OFFSET$ = /(?:[Zz]|[+-]\d\d:\d\d)$/;

const { exec: Z_exec } = theRegExp           (/(([+-])\d\d):(\d\d)$/);

const { exec: OFFSET_DATETIME_exec } = /*#__PURE__*/newRegExp`
	^
	${YMD}
	[Tt ]
	${HMS}
	(?:\.\d{1,3}(\d*?)0*)?
	(?:[Zz]|[+-]${_23_}:${_59_})
	$`.valueOf();

const { exec: OFFSET_DATETIME_ZERO_exec } = /*#__PURE__*/newRegExp`
	^
	${YMD}
	[Tt ]
	${HMS}
	()
	[Zz]
	$`.valueOf();

const { test: IS_LOCAL_DATETIME } = /*#__PURE__*/newRegExp`
	^
	${YMD}
	[Tt ]
	${HMS}
	(?:\.\d+)?
	$`.valueOf();

const { test: IS_LOCAL_DATE } = /*#__PURE__*/newRegExp`
	^
	${YMD}
	$`.valueOf();

const { test: IS_LOCAL_TIME } = /*#__PURE__*/newRegExp`
	^
	${HMS}
	(?:\.\d+)?
	$`.valueOf();

const T = /[ t]/;
const DELIMITER_DOT = /[-T:.]/g;
const DOT_ZERO = /\.?0+$/;
const ZERO = /\.(\d*?)0+$/;
const zeroReplacer = (match        , p1        ) => p1;

const Datetime = /*#__PURE__*/( () => {
	const Datetime = function (            ) {
		return this;
	}                                 ;//expression? :undefined, literal? :undefined, dotValue? :undefined
	//                                > .setTime()
	//                                > .getTime() : Date.parse('T')
	// [Symbol.toPrimitive]('number') > .valueOf()
	//                                > .toISOString()
	const descriptors = Null$1(null)                                         ;
	{
		const descriptor = Null$1(null);
		for ( const key of ownKeys(NativeDate.prototype                                         ) ) {
			key==='constructor' ||
			key==='toJSON' ||
			( descriptors[key] = descriptor );
		}
	}
	Datetime.prototype = preventExtensions(create$1(NativeDate.prototype, descriptors));
	return freeze(Datetime);
} )();

                                        
                                      
                                      
                                      
                                      
                                      
                                       
                                     
                                            
                             
                             

const Value = (ISOString        )        => ISOString.replace(ZERO, zeroReplacer).replace(DELIMITER_DOT, '');

const d = /./gs;
const d2u = (d        ) => '\u2000\u2001\u2002\u2003\u2004\u2005\u2006\u2007\u2008\u2009'[d                                                             ] ;
const ValueOFFSET = (time        , more        )        => time<0
	? ( '' + ( time + 62167305540000 ) ).replace(d, d2u).padStart(14, '\u2000') + more.replace(d, d2u) + time
	: more
		? ( time + '.' ).padStart(16, '0') + more
		: ( '' + time ).padStart(15, '0');

const validateLeap = (literal        )          => {
	if ( literal.startsWith('02-29', 5) ) {
		const year         = +literal.slice(0, 4);
		return (
			year & 0b11 ? false :
				year%100 ? true :
					year%400 ? false :
						year%3200 ? true :
							false
		);
	}
	return true;
};
const { test: VALIDATE_LEAP } = /*#__PURE__*/newRegExp.s`^.....(?:06.30|12.31).23:59:59`.valueOf();

const DATE$1             = /*#__PURE__*/defineProperties(new NativeDate(0), /*#__PURE__*/getOwnPropertyDescriptors(NativeDate.prototype));

const OffsetDateTime_ISOString                = Symbol$1('OffsetDateTime_ISOString')       ;
const OffsetDateTime_value                = Symbol$1('OffsetDateTime_value')       ;
const OffsetDateTime_use = (that                                     , $         = 0) => {
	DATE$1.setTime(+that[OffsetDateTime_value] + $);
	return DATE$1;
};
/*const OffsetDateTime_get = (that :InstanceType<typeof OffsetDateTime>, start :number, end :number) => +that[OffsetDateTime_ISOString].slice(start, end);
const OffsetDateTime_set = (that :InstanceType<typeof OffsetDateTime>, start :number, end :number, value :number, reserveMore :boolean) => {
	if ( end ) {
		const string = '' + value;
		const size = end - start;
		if ( string.length>size ) { throw RangeError(); }///
		that[OffsetDateTime_ISOString] = that[OffsetDateTime_ISOString].slice(0, start) + string.padStart(size, '0') + that[OffsetDateTime_ISOString].slice(end);
	}
	const time = parse(that[OffsetDateTime_ISOString]);
	return that[OffsetDateTime_value] = ValueOFFSET(time, that[OffsetDateTime_value].includes('-')
		? that[OffsetDateTime_value].slice(14, that[OffsetDateTime_value].indexOf('-', 14))
		: that[OffsetDateTime_value].slice(15)
	);///time
};*///
const OffsetDateTime = /*#__PURE__*/fpc(class OffsetDateTime extends Datetime {
	
	[OffsetDateTime_ISOString]        ;
	[OffsetDateTime_value]       ;
	
	get [Symbol$1.toStringTag] () { return 'OffsetDateTime'         ; }
	
	         valueOf (                    )        { return this[OffsetDateTime_value]; }
	toISOString (                    )         { return this[OffsetDateTime_ISOString]; }
	
	constructor (literal        ) {
		validateLeap(literal) || throws(SyntaxError$1(`Invalid Offset Date-Time ${literal}` + where(' at ')));
		const with60 = literal.startsWith('60', 17);
		let without60 = with60 ? literal.slice(0, 17) + '59' + literal.slice(19) : literal;
		const { 1: more = '' } = ( zeroDatetime ? OFFSET_DATETIME_ZERO_exec(without60) : OFFSET_DATETIME_exec(without60) ) || throws(SyntaxError$1(`Invalid Offset Date-Time ${literal}` + where(' at ')));
		const time = parse$2(without60 = without60.replace(T, 'T').replace('z', 'Z'));
		if ( with60 ) {
			DATE$1.setTime(time);
			VALIDATE_LEAP(DATE$1.toISOString()) || throws(SyntaxError$1(`Invalid Offset Date-Time ${literal}` + where(' at ')));
		}
		super();
		this[OffsetDateTime_ISOString] = without60;
		this[OffsetDateTime_value] = ValueOFFSET(time, more);
		return this;
	}
	
	getUTCFullYear (                    )           { return OffsetDateTime_use(this).getUTCFullYear(); }
	///get year () :FullYear { return OffsetDateTime_get(this, 0, 4); }
	///set year (value :FullYear) { OffsetDateTime_set(this, 0, 4, value, true); }
	getUTCMonth (                    )        { return OffsetDateTime_use(this).getUTCMonth(); }
	///get month () { return OffsetDateTime_get(this, 5, 7); }
	///set month (value) { OffsetDateTime_set(this, 5, 7, value, true); }
	getUTCDate (                    )       { return OffsetDateTime_use(this).getUTCDate(); }
	///get day () :Date { return OffsetDateTime_get(this, 8, 10); }
	///set day (value :Date) { OffsetDateTime_set(this, 8, 10, value, true); }
	
	getUTCHours (                    )        { return OffsetDateTime_use(this).getUTCHours(); }
	///get hour () :Hours { return OffsetDateTime_get(this, 11, 13); }
	///set hour (value :Hours) { OffsetDateTime_set(this, 11, 13, value, true); }
	getUTCMinutes (                    )          { return OffsetDateTime_use(this).getUTCMinutes(); }
	///get minute () :Minutes { return OffsetDateTime_get(this, 14, 16); }
	///set minute (value :Minutes) { OffsetDateTime_set(this, 14, 16, value, true); }
	getUTCSeconds (                    )          { return OffsetDateTime_use(this).getUTCSeconds(); }
	///get second () :Seconds { return OffsetDateTime_get(this, 17, 19); }
	///set second (value :Seconds) { OffsetDateTime_set(this, 17, 19, value, true); }
	getUTCMilliseconds (                    )               { return OffsetDateTime_use(this).getUTCMilliseconds(); }///
	///get millisecond () :Milliseconds { return this[OffsetDateTime_value]%1000; }///
	/*set millisecond (value :Milliseconds) {
		this[OffsetDateTime_ISOString] = this[OffsetDateTime_ISOString].slice(0, 19) + ( value ? ( '.' + ( '' + value ).padStart(3, '0') ).replace(DOT_ZERO, '') : '' ) + this[OffsetDateTime_ISOString].slice(this[OffsetDateTime_ISOString].search(OFFSET$));
		OffsetDateTime_set(this, 0, 0, 0, false);
	}*///
	///get microsecond () :Milliseconds
	///set microsecond (value :Milliseconds)
	///get nanosecond () :Milliseconds
	///set nanosecond (value :Milliseconds)
	
	getUTCDay (                    )      { return OffsetDateTime_use(this).getUTCDay(); }
	///get dayOfWeek () { return OffsetDateTime_use(this, this.getTimezoneOffset()*60000).getUTCDay() || 7; }
	getTimezoneOffset (                    )                 {
		const z = Z_exec(this[OffsetDateTime_ISOString]);
		return z ? +z[1]*60 + +( z[2] + z[3] ) : 0;
	}
	///get offset () { return this[OffsetDateTime_ISOString].endsWith('Z') ? 'Z' : this[OffsetDateTime_ISOString].slice(-6); }
	/*set offset (value) {
		this[OffsetDateTime_ISOString] = this[OffsetDateTime_ISOString].slice(0, this[OffsetDateTime_ISOString].endsWith('Z') ? -1 : -6) + value;
		OffsetDateTime_set(this, 0, 0, 0, true);
	}*///
	getTime (                    )       { return floor(+this[OffsetDateTime_value]); }///
	/*setTime (this :OffsetDateTime, value :Time) :void {
		value = DATE.setTime(value);
		const z = Z_exec(this[OffsetDateTime_ISOString]);
		DATE.setTime(value + ( z ? +z[1]*60 + +( z[2] + z[3] ) : 0 )*60000);
		this[OffsetDateTime_ISOString] = z ? DATE.toISOString().slice(0, -1) + z[0] : DATE.toISOString();
		this[OffsetDateTime_value] = ValueOFFSET(value, '');
		///return value;
	}*/
	
});

const LocalDateTime_ISOString                = Symbol$1('LocalDateTime_ISOString')       ;
const LocalDateTime_value                = Symbol$1('LocalDateTime_value')       ;
const LocalDateTime_get = (that                                    , start        , end        ) => +that[LocalDateTime_ISOString].slice(start, end);
const LocalDateTime_set = (that                                    , start        , end        , value        )       => {
	const string = '' + value;
	const size = end - start;
	if ( string.length>size ) { throw RangeError$1(); }///
	that[LocalDateTime_value] = Value(
		that[LocalDateTime_ISOString] = that[LocalDateTime_ISOString].slice(0, start) + string.padStart(size, '0') + that[LocalDateTime_ISOString].slice(end)
	);
};
const LocalDateTime = /*#__PURE__*/fpc(class LocalDateTime extends Datetime {
	
	[LocalDateTime_ISOString]        ;
	[LocalDateTime_value]       ;
	
	get [Symbol$1.toStringTag] () { return 'LocalDateTime'         ; }
	
	         valueOf (                   )        { return this[LocalDateTime_value]; }
	toISOString (                   )         { return this[LocalDateTime_ISOString]; }
	
	constructor (literal        ) {
		IS_LOCAL_DATETIME(literal) && validateLeap(literal) || throws(SyntaxError$1(`Invalid Local Date-Time ${literal}` + where(' at ')));
		super();
		this[LocalDateTime_value] = Value(
			this[LocalDateTime_ISOString] = literal.replace(T, 'T')
		);
		return this;
	}
	
	getFullYear (                   )           { return LocalDateTime_get(this, 0, 4); }
	setFullYear (                     value          )       { LocalDateTime_set(this, 0, 4, value); }
	getMonth (                   )        { return LocalDateTime_get(this, 5, 7) - 1; }
	setMonth (                     value       )       { LocalDateTime_set(this, 5, 7, value + 1); }
	getDate (                   )       { return LocalDateTime_get(this, 8, 10); }
	setDate (                     value      )       { LocalDateTime_set(this, 8, 10, value); }
	
	getHours (                   )        { return LocalDateTime_get(this, 11, 13); }
	setHours (                     value       )       { LocalDateTime_set(this, 11, 13, value); }
	getMinutes (                   )          { return LocalDateTime_get(this, 14, 16); }
	setMinutes (                     value         )       { LocalDateTime_set(this, 14, 16, value); }
	getSeconds (                   )          { return LocalDateTime_get(this, 17, 19); }
	setSeconds (                     value         )       { LocalDateTime_set(this, 17, 19, value); }
	getMilliseconds (                   )               { return +this[LocalDateTime_value].slice(14, 17).padEnd(3, '0'); }///
	setMilliseconds (                     value              )       {
		this[LocalDateTime_value] = Value(
			this[LocalDateTime_ISOString] = this[LocalDateTime_ISOString].slice(0, 19) + ( value ? ( '.' + ( '' + value ).padStart(3, '0') ).replace(DOT_ZERO, '') : '' )
		);
	}
	
});

const LocalDate_ISOString                = Symbol$1('LocalDate_ISOString')       ;
const LocalDate_value                = Symbol$1('LocalDate_value')       ;
const LocalDate_get = (that                                , start        , end        ) => +that[LocalDate_ISOString].slice(start, end);
const LocalDate_set = (that                                , start        , end        , value        )       => {
	const string = '' + value;
	const size = end - start;
	if ( string.length>size ) { throw RangeError$1(); }///
	that[LocalDate_value] = Value(
		that[LocalDate_ISOString] = that[LocalDate_ISOString].slice(0, start) + string.padStart(size, '0') + that[LocalDate_ISOString].slice(end)
	);
};
const LocalDate = /*#__PURE__*/fpc(class LocalDate extends Datetime {
	
	[LocalDate_ISOString]        ;
	[LocalDate_value]       ;
	
	get [Symbol$1.toStringTag] () { return 'LocalDate'         ; }
	
	         valueOf (               )        { return this[LocalDate_value]; }
	toISOString (               )         { return this[LocalDate_ISOString]; }
	
	constructor (literal        ) {
		IS_LOCAL_DATE(literal) && validateLeap(literal) || throws(SyntaxError$1(`Invalid Local Date ${literal}` + where(' at ')));
		super();
		this[LocalDate_value] = Value(
			this[LocalDate_ISOString] = literal
		);
		return this;
	}
	
	getFullYear (               )           { return LocalDate_get(this, 0, 4); }
	setFullYear (                 value          )       { LocalDate_set(this, 0, 4, value); }
	getMonth (               )        { return LocalDate_get(this, 5, 7) - 1; }
	setMonth (                 value       )       { LocalDate_set(this, 5, 7, value + 1); }
	getDate (               )       { return LocalDate_get(this, 8, 10); }
	setDate (                 value      )       { LocalDate_set(this, 8, 10, value); }
	
});

const LocalTime_ISOString                = Symbol$1('LocalTime_ISOString')       ;
const LocalTime_value                = Symbol$1('LocalTime_value')       ;
const LocalTime_get = (that                                , start        , end        ) => +that[LocalTime_ISOString].slice(start, end);
const LocalTime_set = (that                                , start        , end        , value        )       => {
	const string = '' + value;
	const size = end - start;
	if ( string.length>size ) { throw RangeError$1(); }///
	that[LocalTime_value] = Value(
		that[LocalTime_ISOString] = that[LocalTime_ISOString].slice(0, start) + string.padStart(2, '0') + that[LocalTime_ISOString].slice(end)
	);
};
const LocalTime = /*#__PURE__*/fpc(class LocalTime extends Datetime {
	
	[LocalTime_ISOString]        ;
	[LocalTime_value]       ;
	
	get [Symbol$1.toStringTag] () { return 'LocalTime'         ; }
	
	         valueOf (               )        { return this[LocalTime_value]; }
	toISOString (               )         { return this[LocalTime_ISOString]; }
	
	constructor (literal        ) {
		IS_LOCAL_TIME(literal) || throws(SyntaxError$1(`Invalid Local Time ${literal}` + where(' at ')));
		super();
		this[LocalTime_value] = Value(
			this[LocalTime_ISOString] = literal
		);
		return this;
	}
	
	getHours (               )        { return LocalTime_get(this, 0, 2); }
	setHours (                 value       )       { LocalTime_set(this, 0, 2, value); }
	getMinutes (               )          { return LocalTime_get(this, 3, 5); }
	setMinutes (                 value         )       { LocalTime_set(this, 3, 5, value); }
	getSeconds (               )          { return LocalTime_get(this, 6, 8); }
	setSeconds (                 value         )       { LocalTime_set(this, 6, 8, value); }
	getMilliseconds (               )               { return +this[LocalTime_value].slice(6, 9).padEnd(3, '0'); }///
	setMilliseconds (                 value              )       {
		this[LocalTime_value] = Value(
			this[LocalTime_ISOString] = this[LocalTime_ISOString].slice(0, 8) + ( value ? ( '.' + ( '' + value ).padStart(3, '0') ).replace(DOT_ZERO, '') : '' )
		);
	}
	
});

const parseInt$1 = parseInt;

const fromCodePoint = String.fromCodePoint;

const ESCAPED_IN_SINGLE_LINE = /[^\\]+|\\(?:[\\"btnfr/]|u.{4}|U.{8})/gs;
const ESCAPED_IN_MULTI_LINE = /[^\n\\]+|\n|\\(?:[\t ]*\n[\t\n ]*|[\\"btnfr/]|u.{4}|U.{8})/gs;

const BasicString = (literal        )         => {
	if ( !literal ) { return ''; }
	const parts = literal.match(ESCAPED_IN_SINGLE_LINE) ;
	const { length } = parts;
	let index = 0;
	do {
		const part = parts[index] ;
		if ( part[0]==='\\' ) {
			switch ( part[1] ) {
				case '\\': parts[index] = '\\'; break;
				case '"': parts[index] = '"'; break;
				case 'b': parts[index] = '\b'; break;
				case 't': parts[index] = '\t'; break;
				case 'n': parts[index] = '\n'; break;
				case 'f': parts[index] = '\f'; break;
				case 'r': parts[index] = '\r'; break;
				case 'u':
					const charCode         = parseInt$1(part.slice(2), 16);
					mustScalar && 0xD7FF<charCode && charCode<0xE000
					&& throws(RangeError$1(`Invalid Unicode Scalar ${part}` + where(' at ')));
					parts[index] = fromCharCode(charCode);
					break;
				case 'U':
					const codePoint         = parseInt$1(part.slice(2), 16);
					( mustScalar && 0xD7FF<codePoint && codePoint<0xE000 || 0x10FFFF<codePoint )
					&& throws(RangeError$1(`Invalid Unicode Scalar ${part}` + where(' at ')));
					parts[index] = fromCodePoint(codePoint);
					break;
				case '/': parts[index] = '/'; break;
			}
		}
	}
	while ( ++index!==length );
	return parts.join('');
};

const MultilineBasicString = (literal        , useWhatToJoinMultilineString        , n        )         => {
	if ( !literal ) { return ''; }
	const parts = literal.match(ESCAPED_IN_MULTI_LINE) ;
	const { length } = parts;
	let index = 0;
	do {
		const part = parts[index] ;
		if ( part==='\n' ) {
			++n;
			parts[index] = useWhatToJoinMultilineString;
		}
		else if ( part[0]==='\\' ) {
			switch ( part[1] ) {
				case '\n':
				case ' ':
				case '\t':
					for ( let i = 0; i = part.indexOf('\n', i) + 1; ) { ++n; }
					parts[index] = '';
					break;
				case '\\': parts[index] = '\\'; break;
				case '"': parts[index] = '"'; break;
				case 'b': parts[index] = '\b'; break;
				case 't': parts[index] = '\t'; break;
				case 'n': parts[index] = '\n'; break;
				case 'f': parts[index] = '\f'; break;
				case 'r': parts[index] = '\r'; break;
				case 'u':
					const charCode         = parseInt$1(part.slice(2), 16);
					mustScalar && 0xD7FF<charCode && charCode<0xE000
					&& throws(RangeError$1(`Invalid Unicode Scalar ${part}` + where(' at ', lineIndex + n)));
					parts[index] = fromCharCode(charCode);
					break;
				case 'U':
					const codePoint         = parseInt$1(part.slice(2), 16);
					( mustScalar && 0xD7FF<codePoint && codePoint<0xE000 || 0x10FFFF<codePoint )
					&& throws(RangeError$1(`Invalid Unicode Scalar ${part}` + where(' at ', lineIndex + n)));
					parts[index] = fromCodePoint(codePoint);
					break;
				case '/': parts[index] = '/'; break;
			}
		}
	}
	while ( ++index!==length );
	return parts.join('');
};

const INTEGER_D = /[-+]?(?:0|[1-9][_\d]*)/;
const { test: BAD_D } = /*#__PURE__*/newRegExp`_(?!\d)`.valueOf();
const { test: IS_D_INTEGER } = /*#__PURE__*/newRegExp`^${INTEGER_D}$`.valueOf();
const { test: IS_XOB_INTEGER } = theRegExp(/^0(?:x[\dA-Fa-f][_\dA-Fa-f]*|o[0-7][_0-7]*|b[01][_01]*)$/);
const { test: BAD_XOB } = /*#__PURE__*/newRegExp`_(?![\dA-Fa-f])`.valueOf();
const UNDERSCORES$1 = /_/g;
const UNDERSCORES_SIGN = /_|^[-+]/g;

const IS_INTEGER = (literal        )          => ( IS_D_INTEGER(literal) || /*options.xob && */IS_XOB_INTEGER(literal) ) && !BAD_XOB(literal);

const MIN         = BigInt$1 && -/*#__PURE__*/BigInt$1('0x8000000000000000');// -(2n**(64n-1n)) || -MAX-1n
const MAX         = BigInt$1 && /*#__PURE__*/BigInt$1('0x7FFFFFFFFFFFFFFF');// 2n**(64n-1n)-1n || -MIN-1n

const BigIntInteger = (literal        )         => {
	IS_INTEGER(literal) || throws(SyntaxError$1(`Invalid Integer ${literal}` + where(' at ')));
	const bigInt         = literal[0]==='-'
		? -BigInt$1(literal.replace(UNDERSCORES_SIGN, ''))
		: BigInt$1(literal.replace(UNDERSCORES_SIGN, ''));
	allowLonger || MIN<=bigInt && bigInt<=MAX || throws(RangeError$1(`Integer expect 64 bit range (-9,223,372,036,854,775,808 to 9,223,372,036,854,775,807), not includes ${literal}` + where(' meet at ')));
	return bigInt;
};

const NumberInteger = (literal        )         => {
	IS_INTEGER(literal) || throws(SyntaxError$1(`Invalid Integer ${literal}` + where(' at ')));
	const number = parseInt$1(literal.replace(UNDERSCORES$1, ''));
	isSafeInteger(number) || throws(RangeError$1(`Integer did not use BitInt must fit Number.isSafeInteger, not includes ${literal}` + where(' meet at ')));
	return number;
};

const Integer = (literal        )                  => {
	if ( usingBigInt===true ) { return BigIntInteger(literal); }
	if ( usingBigInt===false ) { return NumberInteger(literal); }
	IS_INTEGER(literal) || throws(SyntaxError$1(`Invalid Integer ${literal}` + where(' at ')));
	const number         = parseInt$1(literal.replace(UNDERSCORES$1, ''));
	if ( IntegerMinNumber<=number && number<=IntegerMaxNumber ) { return number; }
	const bigInt         = literal[0]==='-'
		? -BigInt$1(literal.replace(UNDERSCORES_SIGN, ''))
		: BigInt$1(literal.replace(UNDERSCORES_SIGN, ''));
	allowLonger || MIN<=bigInt && bigInt<=MAX || throws(RangeError$1(`Integer expect 64 bit range (-9,223,372,036,854,775,808 to 9,223,372,036,854,775,807), not includes ${literal}` + where(' meet at ')));
	return bigInt;
};

const isFinite$1 = isFinite;

const NaN$1 = 0/0;

const _NaN = -NaN$1;
const _Infinity$1 = -Infinity;
const { test: IS_FLOAT } = /*#__PURE__*/newRegExp`
	^
	${INTEGER_D}
	(?:
		\.\d[_\d]*
		(?:[eE][-+]?\d[_\d]*)?
	|
		[eE][-+]?\d[_\d]*
	)
	$`.valueOf();
const UNDERSCORES = /_/g;
const { test: IS_ZERO } = theRegExp(/^[-+]?0(?:\.0+)?(?:[eE][-+]?0+)?$/);
const { exec: NORMALIZED } = theRegExp   (/^[-0]?(\d*)(?:\.(\d+))?(?:e\+?(-?\d+))?$/);
const { exec: ORIGINAL } = theRegExp   (/^[-+]?0?(\d*)(?:\.(\d*?)0*)?(?:[eE]\+?(-?\d+))?$/);

const Float = (literal        )         => {
	if ( !IS_FLOAT(literal) || BAD_D(literal) ) {
		if ( sFloat ) {
			if ( literal==='inf' || literal==='+inf' ) { return Infinity; }
			if ( literal==='-inf' ) { return _Infinity$1; }
			if ( literal==='nan' || literal==='+nan' ) { return NaN$1; }
			if ( literal==='-nan' ) { return _NaN; }
		}
		else if ( !sError ) {
			if ( literal==='inf' || literal==='+inf' ) { return Infinity; }
			if ( literal==='-inf' ) { return _Infinity$1; }
		}
		throw throws(SyntaxError$1(`Invalid Float ${literal}` + where(' at ')));
	}
	const withoutUnderscores         = literal.replace(UNDERSCORES, '');
	const number         = +withoutUnderscores;
	if ( sError ) {
		isFinite$1(number) || throws(RangeError$1(`Float ${literal} has been as big as inf` + where(' at ')));
		number || IS_ZERO(withoutUnderscores) || throws(RangeError$1(`Float ${literal} has been as little as ${literal[0]==='-' ? '-' : ''}0` + where(' at ')));
		const { 1: normalized_integer, 2: normalized_fractional = '', 3: normalized_exponent = '' } = NORMALIZED(number       ) ;
		const { 1: original_integer, 2: original_fractional = '', 3: original_exponent = '' } = ORIGINAL(withoutUnderscores) ;
		original_integer + original_fractional===normalized_integer + normalized_fractional
		&&
		original_exponent        - original_fractional.length===normalized_exponent        - normalized_fractional.length
		||
		throws(RangeError$1(`Float ${literal} has lost its exact and been ${number}` + where(' at ')));
	}
	return number;
};

const prepareTable = (table       , keys               )        => {
	const { length } = keys;
	let index         = 0;
	while ( index<length ) {
		const key         = keys[index++] ;
		if ( key in table ) {
			table = table[key];
			if ( isTable(table) ) {
				isInline(table) && throws(Error$1(`Trying to define Table under Inline Table` + where(' at ')));
			}
			else if ( isArray(table) ) {
				isStatic(table) && throws(Error$1(`Trying to append value to Static Array` + where(' at ')));
				table = table[( table          ).length - 1];
			}
			else { throw throws(Error$1(`Trying to define Table under non-Table value` + where(' at '))); }
		}
		else {
			table = table[key] = new Table(IMPLICITLY);
			while ( index<length ) { table = table[keys[index++] ] = new Table(IMPLICITLY); }
			return table;
		}
	}
	return table;
};

const appendTable = (table       , finalKey        , asArrayItem         , tag        )        => {
	let lastTable       ;
	if ( asArrayItem ) {
		let arrayOfTables              ;
		if ( finalKey in table ) { isArray(arrayOfTables = table[finalKey]) && !isStatic(arrayOfTables) || throws(Error$1(`Trying to push Table to non-ArrayOfTables value` + where(' at '))); }
		else { arrayOfTables = table[finalKey] = newArray(OF_TABLES); }
		tag && collect(tag, arrayOfTables, table, finalKey);
		arrayOfTables[arrayOfTables.length] = lastTable = new Table(DIRECTLY);
	}
	else {
		if ( finalKey in table ) {
			lastTable = table[finalKey];
			fromPair(lastTable) && throws(Error$1(`A table defined implicitly via key/value pair can not be accessed to via []` + where(', which at ')));
			directlyIfNot(lastTable) || throws(Error$1(`Duplicate Table definition` + where(' at ')));
		}
		else { table[finalKey] = lastTable = new Table(DIRECTLY); }
		tag && collect(tag, null, table, finalKey);
	}
	return lastTable;
};

const prepareInlineTable = (table       , keys          )        => {
	const { length } = keys;
	let index         = 0;
	while ( index<length ) {
		const key         = keys[index++] ;
		if ( key in table ) {
			table = table[key];
			isTable(table) || throws(Error$1(`Trying to assign property through non-Table value` + where(' at ')));
			isInline(table) && throws(Error$1(`Trying to assign property through static Inline Table` + where(' at ')));
			fromPair(table) || throws(Error$1(`A table defined implicitly via [] can not be accessed to via key/value pair` + where(', which at ')));
		}
		else {
			table = table[key] = new Table(IMPLICITLY, PAIR);
			while ( index<length ) { table = table[keys[index++] ] = new Table(IMPLICITLY, PAIR); }
			return table;
		}
	}
	return table;
};

const checkLiteralString = (literal        )         => {
	__CONTROL_CHARACTER_EXCLUDE_test(literal) && throws(SyntaxError$1(`Control characters other than Tab are not permitted in a Literal String` + where(', which was found at ')));
	return literal;
};

const assignLiteralString = ( (table       , finalKey        , literal        )         => {
	if ( !literal.startsWith(`'''`) ) {
		const $ = LITERAL_STRING_exec(literal) || throws(SyntaxError$1(`Bad literal string` + where(' at ')));
		const value = checkLiteralString($[1]);
		table[finalKey] = preserveLiteral ? LiteralObject(literal.slice(0, value.length + 2), value) : value;
		return $[2];
	}
	const $ = __MULTI_LINE_LITERAL_STRING_exec(literal.slice(3));
	if ( $ ) {
		const value = checkLiteralString($[1]) + $[2];
		table[finalKey] = preserveLiteral ? LiteralObject(literal.slice(0, value.length + 6), value) : value;
		return $[3];
	}
	const start = new mark('Multi-line Literal String', literal.length);
	const leadingNewline = !( literal = literal.slice(3) );
	if ( leadingNewline ) {
		literal = start.must();
		const $ = __MULTI_LINE_LITERAL_STRING_exec(literal);
		if ( $ ) {
			const value = checkLiteralString($[1]) + $[2];
			table[finalKey] = preserveLiteral ? LiteralObject([ `'''`, literal.slice(0, value.length + 3) ], value) : value;
			return $[3];
		}
	}
	useWhatToJoinMultilineString===null && start.nowrap(ARGS_MODE);
	for ( const lines                          = [ checkLiteralString(literal) ]; ; ) {
		const line         = start.must();
		const $ = __MULTI_LINE_LITERAL_STRING_exec(line);
		if ( $ ) {
			lines[lines.length] = checkLiteralString($[1]) + $[2];
			const value = lines.join(useWhatToJoinMultilineString );
			if ( preserveLiteral ) {
				lines[lines.length - 1] += `'''`;
				leadingNewline ? lines.unshift(`'''`) : lines[0] = `'''${literal}`;
				table[finalKey] = LiteralObject(lines, value);
			}
			else { table[finalKey] = value; }
			return $[3];
		}
		lines[lines.length] = checkLiteralString(line);
	}
} )     
	                                                                       
	                                                                      
 ;

const assignBasicString = ( (table       , finalKey        , literal        )         => {
	if ( !literal.startsWith('"""') ) {
		const index = BASIC_STRING_exec_1_endIndex(literal);
		const value = BasicString(literal.slice(1, index));
		table[finalKey] = preserveLiteral ? LiteralObject(literal.slice(0, index + 1), value) : value;
		return literal.slice(index + 1).replace(PRE_WHITESPACE, '');
	}
	let length = 3 + MULTI_LINE_BASIC_STRING_exec_0_length(literal.slice(3));
	if ( literal.length!==length ) {
		const $ = literal.slice(3, length);
		ESCAPED_EXCLUDE_CONTROL_CHARACTER_test($) || throws(SyntaxError$1(`Bad multi-line basic string` + where(' at ')));
		const value = BasicString($) + ( literal.startsWith('"', length += 3) ? literal.startsWith('"', ++length) ? ( ++length, '""' ) : '"' : '' );
		table[finalKey] = preserveLiteral ? LiteralObject(literal.slice(0, length), value) : value;
		return literal.slice(length).replace(PRE_WHITESPACE, '');
	}
	const start = new mark('Multi-line Basic String', length);
	const skipped        = ( literal = literal.slice(3) ) ? 0 : 1;
	if ( skipped ) {
		literal = start.must();
		let length = MULTI_LINE_BASIC_STRING_exec_0_length(literal);
		if ( literal.length!==length ) {
			const $ = literal.slice(0, length);
			ESCAPED_EXCLUDE_CONTROL_CHARACTER_test($) || throws(SyntaxError$1(`Bad multi-line basic string` + where(' at ')));
			const value = MultilineBasicString($, useWhatToJoinMultilineString , skipped) + ( literal.startsWith('"', length += 3) ? literal.startsWith('"', ++length) ? ( ++length, '""' ) : '"' : '' );
			table[finalKey] = preserveLiteral ? LiteralObject([ '"""', literal.slice(0, length) ], value) : value;
			return literal.slice(length).replace(PRE_WHITESPACE, '');
		}
	}
	useWhatToJoinMultilineString===null && start.nowrap(ARGS_MODE);
	ESCAPED_EXCLUDE_CONTROL_CHARACTER_test(literal + '\n') || throws(SyntaxError$1(`Bad multi-line basic string` + where(' at ')));
	for ( const lines                          = [ literal ]; ; ) {
		const line         = start.must();
		let length = MULTI_LINE_BASIC_STRING_exec_0_length(line);
		if ( line.length!==length ) {
			const $ = line.slice(0, length);
			ESCAPED_EXCLUDE_CONTROL_CHARACTER_test($) || throws(SyntaxError$1(`Bad multi-line basic string` + where(' at ')));
			const value = MultilineBasicString(lines.join('\n') + '\n' + $, useWhatToJoinMultilineString , skipped) + ( line.startsWith('"', length += 3) ? line.startsWith('"', ++length) ? ( ++length, '""' ) : '"' : '' );
			if ( preserveLiteral ) {
				skipped ? lines.unshift('"""') : lines[0] = `"""${literal}`;
				lines[lines.length] = `${$}"""`;
				table[finalKey] = LiteralObject(lines, value);
			}
			else { table[finalKey] = value; }
			return line.slice(length).replace(PRE_WHITESPACE, '');
		}
		ESCAPED_EXCLUDE_CONTROL_CHARACTER_test(line + '\n') || throws(SyntaxError$1(`Bad multi-line basic string` + where(' at ')));
		lines[lines.length] = line;
	}
} )     
	                                                                       
	                                                                      
 ;

const KEYS = /*#__PURE__*/Null$1        (null);
const commentFor = (key        )         => KEYS[key] || ( KEYS[key] = Symbol$1(key) );
const commentForThis                = Symbol$1('this')       ;

const { test: includesNewline } = theRegExp(/\r?\n/g);
const getCOMMENT = (table                                            , keyComment        )                     => {
	if ( keyComment in table ) {
		const comment = table[keyComment];
		if ( typeof comment!=='string' ) { throw TypeError$1(`the value of comment must be a string, while "${comment===null ? 'null' : typeof comment}" type is found`); }
		if ( includesNewline(comment) ) { throw SyntaxError$1(`the value of comment must be a string and can not include newline`); }
		return ` #${comment}`;///
	}
	return '';
};
const getComment =                    (table                                                                               , key   )                     => key in KEYS ? getCOMMENT(table, KEYS[key] ) : '';

const { test: IS_OFFSET$ } = theRegExp(OFFSET$);
const { test: IS_EMPTY } = theRegExp(/^\[[\t ]*]/);

const parseKeys = (rest        )                                                                => {
	let lineRest         = rest;
	const leadingKeys           = [];
	let lastIndex         = -1;
	for ( ; ; ) {
		lineRest || throws(SyntaxError$1(`Empty bare key` + where(' at ')));
		if ( lineRest[0]==='"' ) {
			const index         = BASIC_STRING_exec_1_endIndex(lineRest);
			KEYS$1.test(leadingKeys[++lastIndex] = BasicString(lineRest.slice(1, index))) || throws(Error$1(`Key not allowed` + where(' at ')));
			lineRest = lineRest.slice(index + 1);
		}
		else {
			const isQuoted = lineRest[0]==='\'';
			const key         = ( ( isQuoted ? __LITERAL_KEY_exec : __BARE_KEY_exec )(lineRest) || throws(SyntaxError$1(`Bad ${isQuoted ? 'literal string' : 'bare'} key` + where(' at '))) )[0];
			lineRest = lineRest.slice(key.length);
			KEYS$1.test(leadingKeys[++lastIndex] = isQuoted ? key.slice(1, -1) : key) || throws(Error$1(`Key not allowed` + where(' at ')));
		}
		if ( IS_DOT_KEY(lineRest) ) { lineRest = lineRest.replace(DOT_KEY, ''); }
		else { break; }
	}
	if ( disableDigit ) {
		const keys = rest.slice(0, -lineRest.length);
		( isAmazing(keys) || enableNull && keys==='null' ) && throws(SyntaxError$1(`Bad bare key disabled by xOptions.string` + where(' at ')));
	}
	if ( disallowEmptyKey ) {
		let index         = lastIndex;
		do { leadingKeys[index]  || throws(SyntaxError$1(`Empty key is not allowed before TOML v0.5` + where(', which at '))); }
		while ( index-- );
	}
	const finalKey         = leadingKeys[lastIndex] ;
	leadingKeys.length = lastIndex;
	return { leadingKeys, finalKey, lineRest };
};

const push = (lastArray       , lineRest        )             => {
	if ( lineRest[0]==='<' ) {
		const { 1: tag } = { 2: lineRest } = _VALUE_PAIR_exec(lineRest) || throws(SyntaxError$1(`Bad tag ` + where(' at ')));
		collect(tag, lastArray, null);
		switch ( lineRest && lineRest[0] ) {
			case ',':
			case ']':
			case '':
			case '#':
				lastArray[lastArray.length] = undefined$1;
				return lineRest;
		}
	}
	switch ( lineRest[0] ) {
		case '\'':
			return assignLiteralString(asStrings(lastArray), lastArray.length, lineRest);
		case '"':
			return assignBasicString(asStrings(lastArray), lastArray.length, lineRest);
		case '{':
			inlineTable || throws(SyntaxError$1(`Inline Table is not allowed before TOML v0.4` + where(', which at ')));
			return equalInlineTable(asTables(lastArray), lastArray.length, lineRest);
		case '[':
			return equalStaticArray(asArrays(lastArray), lastArray.length, lineRest);
	}
	const { 1: literal } = { 2: lineRest } = VALUE_REST_exec(lineRest) || throws(SyntaxError$1(`Bad atom value` + where(' at ')));
	if ( literal==='true' ) { asBooleans(lastArray)[lastArray.length] = true; }
	else if ( literal==='false' ) { asBooleans(lastArray)[lastArray.length] = false; }
	else if ( enableNull && literal==='null' ) { asNulls(lastArray)[lastArray.length] = null; }
	else if ( literal.includes(':') ) {
		if ( literal.includes('-') ) {
			if ( IS_OFFSET$(literal) ) {
				asOffsetDateTimes(lastArray)[lastArray.length] = new OffsetDateTime(literal);
			}
			else {
				moreDatetime || throws(SyntaxError$1(`Local Date-Time is not allowed before TOML v0.5` + where(', which at ')));
				asLocalDateTimes(lastArray)[lastArray.length] = new LocalDateTime(literal);
			}
		}
		else {
			moreDatetime || throws(SyntaxError$1(`Local Time is not allowed before TOML v0.5` + where(', which at ')));
			asLocalTimes(lastArray)[lastArray.length] = new LocalTime(literal);
		}
	}
	else if ( literal.indexOf('-')!==literal.lastIndexOf('-') && literal[0]!=='-' ) {
		moreDatetime || throws(SyntaxError$1(`Local Date is not allowed before TOML v0.5` + where(', which at ')));
		asLocalDates(lastArray)[lastArray.length] = new LocalDate(literal);
	}
	else {
		literal.includes('.') || literal.includes('n') || ( literal.includes('e') || literal.includes('E') ) && !literal.startsWith('0x')
			? asFloats(lastArray)[lastArray.length] = preserveLiteral ? LiteralObject(literal, Float(literal)) : Float(literal)
			: asIntegers(lastArray)[lastArray.length] = preserveLiteral ? LiteralObject(literal, Integer(literal)) : Integer(literal)
		;
	}
	return lineRest;
};

const equalStaticArray = function * (            table       , finalKey        , lineRest        )    {
	const staticArray        = table[finalKey] = newArray(STATICALLY);
	if ( IS_EMPTY(lineRest) ) {
		beInline(staticArray, lineRest[1]===']' ? 0 : 3);
		return lineRest.slice(lineRest.indexOf(']')).replace(SYM_WHITESPACE, '');
	}
	const start = new mark('Static Array', lineRest.length);
	let inline               = lineRest.startsWith('[ ') || lineRest.startsWith('[\t') ? 3 : 0;
	lineRest = lineRest.replace(SYM_WHITESPACE, '');
	while ( !lineRest || lineRest[0]==='#' ) {
		inline = null;
		lineRest = start.must().replace(PRE_WHITESPACE, '');
	}
	if ( lineRest[0]===']' ) {
		inline===null || beInline(staticArray, inline);
		return lineRest.replace(SYM_WHITESPACE, '');
	}
	for ( ; ; ) {
		const rest             = push(staticArray, lineRest);
		lineRest = typeof rest==='string' ? rest : yield rest;
		while ( !lineRest || lineRest[0]==='#' ) {
			inline = null;
			lineRest = start.must().replace(PRE_WHITESPACE, '');
		}
		if ( lineRest[0]===',' ) {
			lineRest = lineRest.replace(SYM_WHITESPACE, '');
			while ( !lineRest || lineRest[0]==='#' ) {
				inline = null;
				lineRest = start.must().replace(PRE_WHITESPACE, '');
			}
			if ( lineRest[0]===']' ) { break; }
		}
		else {
			if ( lineRest[0]===']' ) { break; }
			throw throws(SyntaxError$1(`Unexpect character in static array item value` + where(', which is found at ')));
		}
	}
	inline===null || beInline(staticArray, inline);
	return lineRest.replace(SYM_WHITESPACE, '');
}     
	                                                                   
	                                                                  
 ;

const equalInlineTable = function * (            table       , finalKey        , lineRest        )    {
	const inlineTable        = table[finalKey] = new Table(DIRECTLY, INLINE);
	if ( allowInlineTableMultilineAndTrailingCommaEvenNoComma ) {
		const start = new mark('Inline Table', lineRest.length);
		lineRest = lineRest.replace(SYM_WHITESPACE, '');
		let inline = true;
		for ( ; ; ) {
			while ( !lineRest || lineRest[0]==='#' ) {
				inline = false;
				lineRest = start.must().replace(PRE_WHITESPACE, '');
			}
			if ( lineRest[0]==='}' ) { break; }
			const forComment             = ForComment(inlineTable, lineRest);
			const rest             = assign(forComment);
			lineRest = typeof rest==='string' ? rest : yield rest;
			if ( lineRest ) {
				if ( lineRest[0]==='#' ) {
					if ( preserveComment ) { forComment.table[commentFor(forComment.finalKey)] = lineRest.slice(1); }
					inline = false;
					do { lineRest = start.must().replace(PRE_WHITESPACE, ''); }
					while ( !lineRest || lineRest[0]==='#' );
				}
			}
			else {
				inline = false;
				do { lineRest = start.must().replace(PRE_WHITESPACE, ''); }
				while ( !lineRest || lineRest[0]==='#' );
			}
			if ( lineRest[0]===',' ) { lineRest = lineRest.replace(SYM_WHITESPACE, ''); }
		}
		inline || beInline(inlineTable, false);
	}
	else {
		lineRest = lineRest.replace(SYM_WHITESPACE, '') || throws(SyntaxError$1(`Inline Table is intended to appear on a single line` + where(', which broken at ')));
		if ( lineRest[0]!=='}' ) {
			for ( ; ; ) {
				lineRest[0]==='#' && throws(SyntaxError$1(`Inline Table is intended to appear on a single line` + where(', which broken at ')));
				const rest             = assign(ForComment(inlineTable, lineRest));
				lineRest = ( typeof rest==='string' ? rest : yield rest ) || throws(SyntaxError$1(`Inline Table is intended to appear on a single line` + where(', which broken at ')));
				if ( lineRest[0]==='}' ) { break; }
				if ( lineRest[0]===',' ) {
					lineRest = lineRest.replace(SYM_WHITESPACE, '') || throws(SyntaxError$1(`Inline Table is intended to appear on a single line` + where(', which broken at ')));
					lineRest[0]==='}' && throws(SyntaxError$1(`The last property of an Inline Table can not have a trailing comma` + where(', which was found at ')));
				}
			}
		}
	}
	return lineRest.replace(SYM_WHITESPACE, '');
}     
	                                                                   
	                                                                  
 ;

                                                                                              
const ForComment = (lastInlineTable       , lineRest        )             => {
	const { leadingKeys, finalKey, tag } = { lineRest } = KEY_VALUE_PAIR_exec_groups(parseKeys(lineRest));
	return { table: prepareInlineTable(lastInlineTable, leadingKeys), finalKey, tag, lineRest };
};
const assign = ({ finalKey, tag, lineRest, table }            )             => {
	finalKey in table && throws(Error$1(`Duplicate property definition` + where(' at ')));
	if ( tag ) {
		collect(tag, null, table, finalKey);
		switch ( lineRest && lineRest[0] ) {
			case ',':
			case '}':
			case '':
			case '#':
				table[finalKey] = undefined$1;
				return lineRest;
		}
	}
	switch ( lineRest && lineRest[0] ) {
		case '\'':
			return assignLiteralString(table, finalKey, lineRest);
		case '"':
			return assignBasicString(table, finalKey, lineRest);
		case '{':
			inlineTable || throws(SyntaxError$1(`Inline Table is not allowed before TOML v0.4` + where(', which at ')));
			return equalInlineTable(table, finalKey, lineRest);
		case '[':
			return equalStaticArray(table, finalKey, lineRest);
	}
	const { 1: literal } = { 2: lineRest } = VALUE_REST_exec(lineRest) || throws(SyntaxError$1(`Bad atom value` + where(' at ')));
	if ( literal==='true' ) { table[finalKey] = true; }
	else if ( literal==='false' ) { table[finalKey] = false; }
	else if ( enableNull && literal==='null' ) { table[finalKey] = null; }
	else if ( literal.includes(':') ) {
		if ( literal.includes('-') ) {
			if ( IS_OFFSET$(literal) ) {
				table[finalKey] = new OffsetDateTime(literal);
			}
			else {
				moreDatetime || throws(SyntaxError$1(`Local Date-Time is not allowed before TOML v0.5` + where(', which at ')));
				table[finalKey] = new LocalDateTime(literal);
			}
		}
		else {
			moreDatetime || throws(SyntaxError$1(`Local Time is not allowed before TOML v0.5` + where(', which at ')));
			table[finalKey] = new LocalTime(literal);
		}
	}
	else if ( literal.indexOf('-')!==literal.lastIndexOf('-') && literal[0]!=='-' ) {
		moreDatetime || throws(SyntaxError$1(`Local Date is not allowed before TOML v0.5` + where(', which at ')));
		table[finalKey] = new LocalDate(literal);
	}
	else {
		table[finalKey] = literal.includes('.') || literal.includes('n') || ( literal.includes('e') || literal.includes('E') ) && !literal.startsWith('0x')
			? preserveLiteral ? LiteralObject(literal, Float(literal)) : Float(literal)
			: preserveLiteral ? LiteralObject(literal, Integer(literal)) : Integer(literal)
		;
	}
	return lineRest;
};

const Root = ()        => {
	const rootTable        = new Table;
	let lastSectionTable        = rootTable;
	while ( rest() ) {
		const line         = next().replace(PRE_WHITESPACE, '');
		if ( line ) {
			if ( line[0]==='[' ) {
				const { leadingKeys, finalKey, asArrayItem, tag, lineRest } = TABLE_DEFINITION_exec_groups(line, parseKeys);
				const table        = prepareTable(rootTable, leadingKeys);
				if ( lineRest ) {
					lineRest[0]==='#' || throws(SyntaxError$1(`Unexpect charachtor after table header` + where(' at ')));
				}
				lastSectionTable = appendTable(table, finalKey, asArrayItem, tag);
				preserveComment && lineRest && ( lastSectionTable[commentForThis] = asArrayItem ? lineRest.slice(1) : table[commentFor(finalKey)] = lineRest.slice(1) );
			}
			else if ( line[0]==='#' ) {
				__CONTROL_CHARACTER_EXCLUDE_test(line) && throws(SyntaxError$1(`Control characters other than Tab are not permitted in comments` + where(', which was found at ')));
			}
			else {
				const forComment             = ForComment(lastSectionTable, line);
				let rest             = assign(forComment);
				typeof rest==='string' || ( rest = x        (rest) );
				if ( rest ) {
					rest[0]==='#' || throws(SyntaxError$1(`Unexpect charachtor after key/value pair` + where(' at ')));
					if ( preserveComment ) { forComment.table[commentFor(forComment.finalKey)] = rest.slice(1); }
				}
			}
		}
	}
	return rootTable;
};

const MAX_SAFE_INTEGER = Number.MAX_SAFE_INTEGER;

const DATE = Date.prototype;

const valueOf$2 = String.prototype.valueOf;

const isString = (
	/* j-globals: class.isString (internal) */
	/*#__PURE__*/ function () {
		if ( apply.bind ) {
			var valueOf_apply = apply.bind(valueOf$2);
			return function isString (value) {
				try { valueOf_apply(value); }
				catch (error) { return false; }
				return true;
			};
		}
		return function isString (value) {
			try { valueOf$2.apply(value); }
			catch (error) { return false; }
			return true;
		};
	}()
	/* j-globals: class.isString (internal) */
);

const valueOf$1 = Number.prototype.valueOf;

const isNumber = (
	/* j-globals: class.isNumber (internal) */
	/*#__PURE__*/ function () {
		if ( apply.bind ) {
			var valueOf_apply = apply.bind(valueOf$1);
			return function isNumber (value) {
				try { valueOf_apply(value); }
				catch (error) { return false; }
				return true;
			};
		}
		return function isNumber (value) {
			try { valueOf$1.apply(value); }
			catch (error) { return false; }
			return true;
		};
	}()
	/* j-globals: class.isNumber (internal) */
);

const isBigInt = (
	/* j-globals: class.isBigInt (internal) */
	/*#__PURE__*/ function () {
		if ( typeof BigInt==='function' ) {
			var valueOf_apply = apply.bind(BigInt.prototype.valueOf);
			return function isBigInt (value) {
				try { valueOf_apply(value); }
				catch (error) { return false; }
				return true;
			};
		}
		return function isBigInt () { return false; };
	}()
	/* j-globals: class.isBigInt (internal) */
);

const valueOf = BigInt.prototype.valueOf;

const isBoolean = (
	/* j-globals: class.isBoolean (internal) */
	/*#__PURE__*/ function () {
		if ( apply.bind ) {
			var valueOf_apply = apply.bind(valueOf);
			return function isBoolean (value) {
				try { valueOf_apply(value); }
				catch (error) { return false; }
				return true;
			};
		}
		return function isBoolean (value) {
			try { valueOf.apply(value); }
			catch (error) { return false; }
			return true;
		};
	}()
	/* j-globals: class.isBoolean (internal) */
);

const ESCAPED = /*#__PURE__*/Null$1        ({
	.../*#__PURE__*/fromEntries(/*#__PURE__*/[ ...Array$1(0x20) ].map((_, charCode) => [ fromCharCode(charCode), '\\u' + charCode.toString(16).toUpperCase().padStart(4, '0') ])),
	'\b': '\\b',
	'\t': '\\t',
	'\n': '\\n',
	'\f': '\\f',
	'\r': '\\r',
	'"': '\\"',
	'"""': '""\\"',
	'\\': '\\\\',
	'\x7F': '\\u007F',
});

const { test: NEED_BASIC } = theRegExp(/[\x00-\x08\x0A-\x1F'\x7F]/);
const BY_ESCAPE = /[^\x00-\x08\x0A-\x1F"\\\x7F]+|./gs;
const { test: NEED_ESCAPE } = theRegExp(/^[\x00-\x08\x0A-\x1F"\\\x7F]/);
const singlelineString = (value        )                                => {
	if ( NEED_BASIC(value) ) {
		const parts = value.match(BY_ESCAPE) ;
		let index = parts.length;
		do { if ( NEED_ESCAPE(parts[--index] ) ) { parts[index] = ESCAPED[parts[index] ] ; } }
		while ( index );
		return `"${parts.join('')}"`;
	}
	return `'${value}'`;
};
const singlelineBasicString = (value        )                => {
	if ( value ) {
		const parts = value.match(BY_ESCAPE) ;
		let index = parts.length;
		do { if ( NEED_ESCAPE(parts[--index] ) ) { parts[index] = ESCAPED[parts[index] ] ; } }
		while ( index );
		return `"${parts.join('')}"`;
	}
	return `""`;
};

const { test: NEED_MULTILINE_BASIC } = theRegExp(/[\x00-\x08\x0A-\x1F\x7F]|'''/);
const { test: multilineNeedBasic } = theRegExp(/[\x00-\x08\x0B-\x1F\x7F]|'''/);
const { test: REAL_MULTILINE_ESCAPE } = theRegExp(/[\x00-\x08\x0A-\x1F\\\x7F]|"""/);
const BY_MULTILINE_ESCAPE = /[^\x00-\x08\x0A-\x1F"\\\x7F]+|"""|./gs;
const { test: NEED_MULTILINE_ESCAPE } = theRegExp(/^(?:[\x00-\x08\x0A-\x1F\\\x7F]|""")/);
const escape_multiline = (lines          , lineIndex        ) => {
	const line = lines[lineIndex] ;
	if ( REAL_MULTILINE_ESCAPE(line) ) {
		const parts = line.match(BY_MULTILINE_ESCAPE) ;
		let index = parts.length;
		do { if ( NEED_MULTILINE_ESCAPE(parts[--index] ) ) { parts[index] = ESCAPED[parts[index] ] ; } }
		while ( index );
		lines[lineIndex] = parts.join('');
	}
};

                                                    
const Lines = (lines                   )        => ( lines = [ '', ...lines ]          ).length===1 ? [ '', '' ] : lines         ;

const multilineString = (lines       )                                                                                  => {
	const lastIndex = lines.length - 1;
	let index = lastIndex;
	do { if ( NEED_MULTILINE_BASIC(lines[index] ) ) { break; } }
	while ( --index );
	if ( index ) {
		index = lastIndex;
		escape_multiline(lines, index);
		lines[index] += lines[0] = '"""';
		while ( --index ) { escape_multiline(lines, index); }
	}
	else { lines[lastIndex] += lines[0] = '\'\'\''; }
	return lines                                                                                   ;
};

const multilineBasicString = (lines       )                                         => {
	let index = lines.length - 1;
	escape_multiline(lines, index);
	lines[index] += lines[0] = '"""';
	while ( --index ) { escape_multiline(lines, index); }
	return lines                                          ;
};

const multilineLiteralString = (lines       )                                         => {
	lines[lines.length - 1] += lines[0] = '\'\'\'';
	return lines                                          ;
};

const Float64Array$1 = Float64Array;

const Uint8Array$1 = Uint8Array;

const _Infinity = -Infinity;

const { test: INTEGER_LIKE } = theRegExp(/^-?\d+$/);
const ensureFloat = (literal        ) => INTEGER_LIKE(literal) ? literal + '.0' : literal;

const float64Array = new Float64Array$1([ NaN$1 ]);
const uint8Array = new Uint8Array$1(float64Array.buffer);
const NaN_7 = uint8Array[7] ;

const float = NaN_7===new Uint8Array$1(new Float64Array$1([ -NaN$1 ]).buffer)[7] 
	? (value        ) => value
		? value===Infinity ? 'inf' : value===_Infinity ? '-inf' : ensureFloat('' + value)
		: value===value ? is(value, 0) ? '0.0' : '-0.0' : 'nan'
	: (value        ) => value
		? value===Infinity ? 'inf' : value===_Infinity ? '-inf' : ensureFloat('' + value)
		: value===value ? is(value, 0) ? '0.0' : '-0.0' : ( float64Array[0] = value, uint8Array[7] )===NaN_7 ? 'nan' : '-nan';

const isDate = /*#__PURE__*/isPrototypeOf.bind(DATE)                                                ;

const { test: BARE } = theRegExp(/^[\w-]+$/);
const $Key$ = (key        )         => BARE(key) ? key : singlelineString(key);

const FIRST = /[^.]+/;
const literalString = (value        )                => `'${value}'`;
const $Keys = (keys        )         => isAmazing(keys) ? keys.replace(FIRST, literalString) : keys==='null' ? `'null'` : keys;

class TOMLSection extends Array$1         {
	
	                 document              ;
	
	constructor (document              ) {
		super();
		this.document = document;
		return this;
	}
	
	[Symbol$1.toPrimitive] () { return this.join(this.document.newline); }
	
	appendNewline () { this[this.length] = ''; }
	        set appendLine (source        ) { this[this.length] = source; }
	        set appendInline (source        ) { this[this.length - 1] += source; }   
	        set appendInlineIf (source        ) { source && ( this[this.length - 1] += source ); }///
	
	* assignBlock                           (documentKeys_                   , sectionKeys_                  , table   , tableKeys                            )    {
		const { document } = this;
		const { newlineUnderHeader, newlineUnderSectionButPair } = document;
		const newlineAfterDotted = sectionKeys_ ? document.newlineUnderPairButDotted : false;
		const newlineAfterPair = sectionKeys_ ? document.newlineUnderDotted : document.newlineUnderPair;
		for ( const tableKey of tableKeys ) {
			const value                 = table[tableKey] ;
			const $key$ = $Key$(tableKey);
			const documentKeys = documentKeys_ + $key$;
			if ( isArray$1(value) ) {
				const { length } = value;
				if ( length ) {
					let firstItem = value[0];
					if ( isSection(firstItem) ) {
						const tableHeader = `[[${documentKeys}]]`         ;
						const documentKeys_ = documentKeys + '.'                ;
						let index = 0;
						let table                 = firstItem;
						for ( ; ; ) {
							const section = document.appendSection();
							section[0] = tableHeader + getCOMMENT(table, commentForThis);
							if ( newlineUnderHeader ) {
								section[1] = '';
								yield section.assignBlock(documentKeys_, ``, table, getOwnPropertyNames(table));
								newlineUnderSectionButPair && section.length!==2 && section.appendNewline();
							}
							else {
								yield section.assignBlock(documentKeys_, ``, table, getOwnPropertyNames(table));
								newlineUnderSectionButPair && section.appendNewline();
							}
							if ( ++index===length ) { break; }
							table = ( value                           )[index] ;
							if ( !isSection(table) ) { throw TypeError$1(`the first table item marked by Section() means the parent array is an array of tables, which can not include other types or table not marked by Section() any more in the rest items`); }
						}
						continue;
					}
					else { let index = 1; while ( index!==length ) { if ( isSection(value[index++] ) ) { throw TypeError$1(`if an array is not array of tables, it can not include any table that marked by Section()`); } } }
				}
			}
			else {
				if ( isSection(value) ) {
					const section = document.appendSection();
					section[0] = `[${documentKeys}]${
						document.preferCommentForThis
							? getCOMMENT(value, commentForThis) || getComment(table, tableKey)
							: getComment(table, tableKey) || getCOMMENT(value, commentForThis)
					}`;
					if ( newlineUnderHeader ) {
						section[1] = '';
						yield section.assignBlock(documentKeys + '.'                , ``, value, getOwnPropertyNames(value));
						newlineUnderSectionButPair && section.length!==2 && section.appendNewline();
					}
					else {
						yield section.assignBlock(documentKeys + '.'                , ``, value, getOwnPropertyNames(value));
						newlineUnderSectionButPair && section.appendNewline();
					}
					continue;
				}
			}
			const sectionKeys = sectionKeys_ + $key$;
			this.appendLine = $Keys(sectionKeys) + ' = ';
			const valueKeysIfValueIsDottedTable = this.value('', value, true);
			if ( valueKeysIfValueIsDottedTable ) {
				--this.length;
				yield this.assignBlock(documentKeys + '.'                , sectionKeys + '.'                , value                                   , valueKeysIfValueIsDottedTable);
				newlineAfterDotted && this.appendNewline();
			}
			else {
				this.appendInlineIf = getComment(table, tableKey);
				newlineAfterPair && this.appendNewline();
			}
		}
	}
	
	        value (indent        , value                , returnValueKeysIfValueIsDottedTable         )                  {
		switch ( typeof value ) {
			case 'object':
				if ( value===null ) {
					if ( this.document.nullDisabled ) { throw TypeError$1(`toml can not stringify "null" type value without truthy options.xNull`); }
					this.appendInline = 'null';
					break;
				}
				const inlineMode = ofInline(value);
				if ( isArray$1(value) ) {
					if ( inlineMode===undefined$1 ) { this.staticArray(indent, value); }
					else {
						const { $singlelineArray = inlineMode } = this.document;
						this.singlelineArray(indent, value, $singlelineArray);
					}
					break;
				}
				if ( inlineMode!==undefined$1 ) {
					inlineMode || this.document.multilineTableDisabled
						? this.inlineTable(indent, value                        )
						: this.multilineTable(indent, value                        , this.document.multilineTableComma);
					break;
				}
				if ( isDate(value) ) {
					this.appendInline = value.toISOString().replace('T', this.document.T).replace('Z', this.document.Z);
					break;
				}
				if ( _literal in value ) {
					const literal = ( value                                                                       )[_literal];
					if ( typeof literal==='string' ) { this.appendInline = literal; }
					else if ( isArray$1(literal) ) {
						const { length } = literal;
						if ( length ) {
							this.appendInline = literal[0];
							let index = 1;
							while ( index!==length ) { this.appendLine = literal[index++] ; }
						}
						else { throw TypeError$1(`literal value is broken`); }
					}
					else { throw TypeError$1(`literal value is broken`); }
					break;
				}
				if ( isString(value) ) { throw TypeError$1(`TOML.stringify refuse to handle [object String]`); }
				if ( isNumber(value) ) { throw TypeError$1(`TOML.stringify refuse to handle [object Number]`); }
				if ( isBigInt(value) ) { throw TypeError$1(`TOML.stringify refuse to handle [object BigInt]`); }
				if ( isBoolean(value) ) { throw TypeError$1(`TOML.stringify refuse to handle [object Boolean]`); }
				if ( returnValueKeysIfValueIsDottedTable ) {
					const keys = getOwnPropertyNames(value                        );
					if ( keys.length ) { return keys; }
					this.appendInline = '{ }';
				}
				else {
					this.inlineTable(indent, value                        );
				}
				break;
			case 'bigint':
				this.appendInline = '' + value;
				break;
			case 'number':
				this.appendInline = this.document.asInteger(value) ? is(value, -0) ? '-0' : '' + value : float(value);
				break;
			case 'string':
				this.appendInline = singlelineString(value);
				break;
			case 'boolean':
				this.appendInline = value ? 'true' : 'false';
				break;
			default:
				throw TypeError$1(`toml can not stringify "${typeof value}" type value`);
		}
		return null;
	}
	
	        singlelineArray (indent        , staticArray                      , inlineMode               ) {
		const { length } = staticArray;
		if ( length ) {
			this.appendInline = inlineMode&0b10 ? '[ ' : '[';
			this.value(indent, staticArray[0] , false);
			let index = 1;
			while ( index!==length ) {
				this.appendInline = ', ';
				this.value(indent, staticArray[index++] , false);
			}
			this.appendInline = inlineMode&0b10 ? ' ]' : ']';
		}
		else { this.appendInline = inlineMode&0b01 ? '[ ]' : '[]'; }
	}
	        staticArray (indent        , staticArray                      ) {
		this.appendInline = '[';
		const indent_ = indent + this.document.indent;
		const { length } = staticArray;
		let index = 0;
		while ( index!==length ) {
			this.appendLine = indent_;
			this.value(indent_, staticArray[index++] , false);
			this.appendInline = ',';
		}
		this.appendLine = indent + ']';
	}
	
	        inlineTable (indent        , inlineTable                      ) {
		const keys = getOwnPropertyNames(inlineTable);
		if ( keys.length ) {
			this.appendInline = '{ ';
			this.assignInline(indent, inlineTable, ``, keys);
			this[this.length - 1] = this[this.length - 1] .slice(0, -2) + ' }';
		}
		else { this.appendInline = '{ }'; }
	}
	        multilineTable (indent        , inlineTable                      , comma                     ) {
		this.appendInline = '{';
		this.assignMultiline(indent, inlineTable, ``, getOwnPropertyNames(inlineTable), comma);
		this.appendLine = indent + '}';
	}
	        assignInline                                 (indent        , inlineTable   , keys_                   , keys                            ) {
		for ( const key of keys ) {
			const value                 = inlineTable[key] ;
			const keys = keys_ + $Key$(key);
			const before_value = this.appendInline = $Keys(keys) + ' = ';
			const valueKeysIfValueIsDottedTable = this.value(indent, value, true);
			if ( valueKeysIfValueIsDottedTable ) {
				this[this.length - 1] = this[this.length - 1] .slice(0, -before_value.length);
				this.assignInline(indent, value                        , keys + '.'                , valueKeysIfValueIsDottedTable);
			}
			else { this.appendInline = ', '; }
		}
	}
	        assignMultiline                                 (indent        , inlineTable   , keys_                   , keys                            , comma                     ) {
		const indent_ = indent + this.document.indent;
		for ( const key of keys ) {
			const value                 = inlineTable[key] ;
			const keys = keys_ + $Key$(key);
			this.appendLine = indent_ + $Keys(keys) + ' = ';
			const valueKeysIfValueIsDottedTable = this.value(indent_, value, true);
			if ( valueKeysIfValueIsDottedTable ) {
				--this.length;
				this.assignMultiline(indent, value                        , keys + '.'                , valueKeysIfValueIsDottedTable, comma);
			}
			else {
				comma
					? this.appendInline = ',' + getComment(inlineTable, key)
					: this.appendInlineIf = getComment(inlineTable, key);
			}
		}
	}
	
}

const name2code = /*#__PURE__*/Null$1({
	document: 0,
	section: 1,
	header: 2,
	pairs: 3,
	pair: 4,
}         );

const { test: IS_INDENT } = theRegExp(/^[\t ]*$/);

const return_false = () => false;

class TOMLDocument extends Array$1              {
	
	         get ['constructor'] () { return Array$1; }
	
	0 = new TOMLSection(this);
	
	         asInteger                                          = return_false;
	         newline                     = '';
	         newlineUnderSection          = true;
	         newlineUnderSectionButPair          = true;
	         newlineUnderHeader          = true;
	         newlineUnderPair          = false;
	         newlineUnderPairButDotted          = false;
	         newlineUnderDotted          = false;
	         indent         = '\t';
	         T                  = 'T';
	         Z            = 'Z';
	         nullDisabled          = true;
	         multilineTableDisabled          = true;
	         multilineTableComma          ;
	         preferCommentForThis          = false;
	         $singlelineArray                ;
	
	constructor (options                  ) {
		
		super();
		
		if ( options==null ) { return this; }
		
		const { integer } = options;
		if ( integer===undefined ) ;
		else if ( integer===MAX_SAFE_INTEGER ) { this.asInteger = isSafeInteger; }
		else if ( typeof integer==='number' ) {
			if ( !isSafeInteger(integer) ) { throw RangeError$1(`TOML.stringify(,{integer}) can only be a safe integer`); }
			const max = integer>=0 ? integer : -integer - 1;
			const min = integer>=0 ? -integer : integer;
			this.asInteger = (number        ) => isSafeInteger(number) && min<=number && number<=max;
		}
		else { throw TypeError$1(`TOML.stringify(,{integer}) can only be number`); }
		
		const { newline } = options;
		if ( newline===undefined ) ;
		else if ( newline==='\n' || newline==='\r\n' ) { this.newline = newline; }
		else {
			throw typeof newline==='string'
				? SyntaxError$1(`TOML.stringify(,{newline}) can only be valid TOML newline`)
				: TypeError$1(`TOML.stringify(,{newline}) can only be string`);
		}
		
		const { preferCommentFor } = options;
		if ( preferCommentFor===undefined ) ;
		else if ( preferCommentFor==='this' || preferCommentFor==='key' ) { this.preferCommentForThis = preferCommentFor==='this'; }
		else { throw TypeError$1(`TOML.stringify(,{preferCommentFor) can only be 'key' or 'this'`); }
		
		const { [options.newlineAround || 'header']: around = name2code.header } = name2code;
		this.newlineUnderSection = around>0;
		this.newlineUnderSectionButPair = around===1 || around===2;
		this.newlineUnderHeader = around>1;
		this.newlineUnderPair = around>2;
		this.newlineUnderPairButDotted = around===3;
		this.newlineUnderDotted = around>3;
		
		const { indent } = options;
		if ( indent===undefined ) ;
		else if ( typeof indent==='string' ) {
			if ( !IS_INDENT(indent) ) { throw SyntaxError$1(`TOML.stringify(,{indent}) can only include Tab or Space`); }
			this.indent = indent;
		}
		else if ( typeof indent==='number' ) {
			if ( !isSafeInteger(indent) ) { throw RangeError$1(`TOML.stringify(,{indent:${indent}}) is out of range`); }
			this.indent = ' '.repeat(indent);
		}
		else { throw TypeError$1(`TOML.stringify(,{indent}) can not be "${typeof indent}" type`); }
		
		const { T } = options;
		if ( T===undefined ) ;
		else if ( T===' ' || T==='t' || T==='T' ) { this.T = T; }
		else { throw TypeError$1(`TOML.stringify(,{T}) can only be "T" or " " or "t"`); }
		
		const { Z } = options;
		if ( Z===undefined ) ;
		else if ( Z==='z' || Z==='Z' ) { this.Z = Z; }
		else { throw TypeError$1(`TOML.stringify(,{Z}) can only be "Z" or "z"`); }
		
		if ( options.xNull ) { this.nullDisabled = false; }
		
		const { xBeforeNewlineInMultilineTable } = options;
		if ( xBeforeNewlineInMultilineTable===undefined ) ;
		else if ( xBeforeNewlineInMultilineTable==='' || xBeforeNewlineInMultilineTable===',' ) {
			this.multilineTableDisabled = false;
			this.multilineTableComma = !!xBeforeNewlineInMultilineTable;
		}
		else { throw TypeError$1(`TOML.stringify(,{xBeforeNewlineInMultilineTable}) can only be "" or ","`); }
		
		const $singlelineArray = options.forceInlineArraySpacing;
		switch ( $singlelineArray ) {
			case undefined:
				break;
			case 0:
			case 1:
			case 2:
			case 3:
				this.$singlelineArray = $singlelineArray;
				break;
			default:
				throw typeof $singlelineArray==='number'
					? RangeError$1(`array inline mode must be 0 | 1 | 2 | 3, not including ${$singlelineArray}`)
					: TypeError$1(`array inline mode must be "number" type, not including ${$singlelineArray===null ? '"null"' : typeof $singlelineArray}`);
		}
		
		return this;
		
	}
	
	appendSection () { return this[this.length] = new TOMLSection(this); }
	
}

const linesFromStringify = new WeakSet$1                   ();
const beLinesFromStringify = /*#__PURE__*/add.bind(linesFromStringify);
const isLinesFromStringify = /*#__PURE__*/has.bind(linesFromStringify);
const stringify = (rootTable                , options                  )                    => {
	const document = new TOMLDocument(options);
	const section = document[0];
	section[0] = '';
	x      (section.assignBlock(``, ``, rootTable, getOwnPropertyNames(rootTable)));
	document.newlineUnderSectionButPair && section.length!==1 && section.appendNewline();
	document.newlineUnderSection || document[document.length - 1] .appendNewline();
	if ( document.newline ) { return document.join(document.newline); }
	const lines = document.flat();
	beLinesFromStringify(lines);
	return lines;
};
const multiline = /*#__PURE__*/( () => {
	const multiline = (value                                                   , string         ) =>
		typeof value==='string' ? LiteralObject(( multilineNeedBasic(value) ? multilineBasicString : multilineLiteralString )(( '\n' + value ).split('\n')         ), value) :
			isArray$1(value) ? LiteralObject(multilineString(Lines(value)), typeof string==='string' ? string : Null$1(null)) :
				multilineTable(value);
	multiline.basic = (lines                            , string         ) =>
		typeof lines==='string'
			? LiteralObject(multilineBasicString(( '\n' + lines ).split('\n')         ), lines)
			: LiteralObject(multilineBasicString(Lines(lines)), typeof string==='string' ? string : Null$1(null))
	;
	multiline.array = multilineArray;
	freeze(multiline);
	return multiline;
} )();
const basic = (value        ) => LiteralObject(singlelineBasicString(value), value);
const literal = (literal                               , ...chars          ) => {
	if ( typeof literal==='string' ) {
		if ( chars.length===1 ) {
			return LiteralObject(literal.includes('\n') ? literal.split('\n')                            : literal, chars[0]                            );
		}
	}
	else {
		let index = chars.length;
		if ( index ) {
			const { raw } = literal;
			literal = raw[index] ;
			while ( index ) { chars[--index] += raw[index] ; }
			literal = chars.join('') + literal;
		}
		else { literal = literal.raw[0] ; }
	}
	return LiteralObject(literal.includes('\n') ? literal.split('\n')                            : literal, Null$1(null));
};

const textDecoder = /*#__PURE__*/new TextDecoder$1('utf-8', Null$1({ fatal: true, ignoreBOM: false }));
const binary2string = (arrayBufferLike                          )         => {
	if ( isView(arrayBufferLike) ? arrayBufferLike.length!==arrayBufferLike.byteLength : !isArrayBuffer(arrayBufferLike) ) { throw TypeError$1(`only Uint8Array or ArrayBuffer is acceptable`); }
	try { return textDecoder.decode(arrayBufferLike); }
	catch { throw Error$1(`A TOML doc must be a (ful-scalar) valid UTF-8 file, without any unknown code point.`); }
};
const isBinaryLike = (value        )                                    => 'byteLength' in value;///

const { test: includesNonScalar } = theRegExp(/[\uD800-\uDFFF]/u);
const assertFulScalar = (string        )       => {
	if ( clearRegExp$1(includesNonScalar(string)) ) { throw Error$1(`A TOML doc must be a (ful-scalar) valid UTF-8 file, without any uncoupled UCS-4 character code.`); }
};

let holding          = false;

const parse = (source        , specificationVersion                                   , multilineStringJoiner                                                                                                                       , bigint                                       , x                              , argsMode                 )        => {
	let sourcePath         = '';
	if ( typeof source==='object' && source ) {
		if ( isArray$1(source) ) { throw TypeError$1(isLinesFromStringify(source) ? `TOML.parse(array from TOML.stringify(,{newline?}))` : `TOML.parse(array)`); }
		else if ( isBinaryLike(source) ) { source = binary2string(source); }
		else {
			sourcePath = source.path;
			if ( typeof sourcePath!=='string' ) { throw TypeError$1(`TOML.parse(source.path)`); }
			const { data, require: req = typeof require==='function' ? require : undefined$1 } = source;
			if ( req ) {
				const { resolve } = req;
				if ( resolve!=null ) {
					const { paths } = resolve;
					if ( paths!=null ) {
						const ret = apply$1(paths, resolve, [ '' ]);
						if ( ret!=null ) {
							const val = ret[0];
							if ( val!=null ) {
								const dirname_ = val.replace(/node_modules$/, '');
								if ( dirname_ ) {
									sourcePath = ( req                                          )('path').resolve(dirname_, sourcePath);
									if ( typeof sourcePath!=='string' ) { throw TypeError$1(`TOML.parse(source.require('path').resolve)`); }
								}
							}
						}
					}
				}
				if ( data===undefined$1 ) {
					const data = ( req                                      )('fs').readFileSync(sourcePath);
					if ( typeof data==='object' && data && isBinaryLike(data) ) { source = binary2string(data); }
					else { throw TypeError$1(`TOML.parse(source.require('fs').readFileSync)`); }
				}
				else if ( typeof data==='string' ) { assertFulScalar(source = data); }
				else if ( typeof data==='object' && data && isBinaryLike(data) ) { source = binary2string(data); }
				else { throw TypeError$1(`TOML.parse(source.data)`); }
			}
			else {
				if ( data===undefined$1 ) { throw TypeError$1(`TOML.parse(source.data|source.require)`); }
				else if ( typeof data==='string' ) { assertFulScalar(source = data); }
				else if ( typeof data==='object' && data && isBinaryLike(data) ) { source = binary2string(data); }
				else { throw TypeError$1(`TOML.parse(source.data)`); }
			}
		}
	}
	else if ( typeof source==='string' ) { assertFulScalar(source); }
	else { throw TypeError$1(`TOML.parse(source)`); }
	let joiner                    ;
	let keys                                 ;
	if ( typeof multilineStringJoiner==='object' && multilineStringJoiner ) {
		if ( bigint!==undefined$1 || x!==undefined$1 ) { throw TypeError$1(`options mode ? args mode`); }
		joiner = multilineStringJoiner.joiner;
		bigint = multilineStringJoiner.bigint;
		keys = multilineStringJoiner.keys;
		x = multilineStringJoiner.x;
		argsMode = '';
	}
	else { joiner = multilineStringJoiner; }
	let rootTable       ;
	let process                 ;
	if ( holding ) { throw Error$1(`parsing during parsing.`); }
	holding = true;
	try {
		use(specificationVersion, joiner, bigint, keys, x, argsMode);
		todo(source, sourcePath);
		source && source[0]==='\uFEFF' && throws(TypeError$1(`TOML content (string) should not start with BOM (U+FEFF)` + where(' at ')));
		rootTable = Root();
		process = Process();
	}
	finally {
		done();//clearWeakSets();
		clear();
		holding = false;
		clearRegExp$1();
	}
	process && process();
	return rootTable;
};

const parse$1 = /*#__PURE__*/assign$1(
	(source        , specificationVersion                                   , multilineStringJoiner         , useBigInt                   , xOptions                   ) =>
		typeof specificationVersion==='number'
			? parse(source, specificationVersion, multilineStringJoiner, useBigInt, xOptions, ',,')
			: parse(source, 1.0, specificationVersion          , multilineStringJoiner                                       , useBigInt                    , ',')
	,
	{
		'1.0': (source        , multilineStringJoiner         , useBigInt                   , xOptions                   ) => parse(source, 0.1, multilineStringJoiner, useBigInt, xOptions, ','),
		1.0: (source        , multilineStringJoiner         , useBigInt                   , xOptions                   ) => parse(source, 1.0, multilineStringJoiner, useBigInt, xOptions, ','),
		0.5: (source        , multilineStringJoiner         , useBigInt                   , xOptions                   ) => parse(source, 0.5, multilineStringJoiner, useBigInt, xOptions, ','),
		0.4: (source        , multilineStringJoiner         , useBigInt                   , xOptions                   ) => parse(source, 0.4, multilineStringJoiner, useBigInt, xOptions, ','),
		0.3: (source        , multilineStringJoiner         , useBigInt                   , xOptions                   ) => parse(source, 0.3, multilineStringJoiner, useBigInt, xOptions, ','),
		0.2: (source        , multilineStringJoiner         , useBigInt                   , xOptions                   ) => parse(source, 0.2, multilineStringJoiner, useBigInt, xOptions, ','),
		0.1: (source        , multilineStringJoiner         , useBigInt                   , xOptions                   ) => parse(source, 0.1, multilineStringJoiner, useBigInt, xOptions, ','),
	}
);

const _export = /*#__PURE__*/Default({
	version,
	parse: parse$1,
	stringify,
	Section, inline, multiline, basic, literal, commentFor, commentForThis,
	OffsetDateTime, LocalDateTime, LocalDate, LocalTime,
	isInline, isSection,
	Keys,
});



//# sourceMappingURL=index.mjs.map

/***/ }),

/***/ "./noir_wasm/cjs/index_bg.wasm":
/*!*************************************!*\
  !*** ./noir_wasm/cjs/index_bg.wasm ***!
  \*************************************/
/***/ ((module, exports, __webpack_require__) => {

"use strict";
/* harmony import */ var WEBPACK_IMPORTED_MODULE_0 = __webpack_require__(/*! ./index_bg.js */ "./noir_wasm/cjs/index_bg.js");
module.exports = __webpack_require__.v(exports, module.id, "09cbcb80c41921303ad7", {
	"./index_bg.js": {
		"__wbindgen_object_drop_ref": WEBPACK_IMPORTED_MODULE_0.__wbindgen_object_drop_ref,
		"__wbg_constructor_2f52aa89ce97ba46": WEBPACK_IMPORTED_MODULE_0.__wbg_constructor_2f52aa89ce97ba46,
		"__wbg_constructor_1292ee4141d8f79d": WEBPACK_IMPORTED_MODULE_0.__wbg_constructor_1292ee4141d8f79d,
		"__wbg_readfile_698fcfe83a414130": WEBPACK_IMPORTED_MODULE_0.__wbg_readfile_698fcfe83a414130,
		"__wbindgen_is_undefined": WEBPACK_IMPORTED_MODULE_0.__wbindgen_is_undefined,
		"__wbindgen_string_new": WEBPACK_IMPORTED_MODULE_0.__wbindgen_string_new,
		"__wbg_new_abda76e883ba8a5f": WEBPACK_IMPORTED_MODULE_0.__wbg_new_abda76e883ba8a5f,
		"__wbg_stack_658279fe44541cf6": WEBPACK_IMPORTED_MODULE_0.__wbg_stack_658279fe44541cf6,
		"__wbg_error_f851667af71bcfc6": WEBPACK_IMPORTED_MODULE_0.__wbg_error_f851667af71bcfc6,
		"__wbg_debug_efabe4eb183aa5d4": WEBPACK_IMPORTED_MODULE_0.__wbg_debug_efabe4eb183aa5d4,
		"__wbg_error_a7e23606158b68b9": WEBPACK_IMPORTED_MODULE_0.__wbg_error_a7e23606158b68b9,
		"__wbg_error_50f42b952a595a23": WEBPACK_IMPORTED_MODULE_0.__wbg_error_50f42b952a595a23,
		"__wbg_info_24d8f53d98f12b95": WEBPACK_IMPORTED_MODULE_0.__wbg_info_24d8f53d98f12b95,
		"__wbg_log_9b164efbe6db702f": WEBPACK_IMPORTED_MODULE_0.__wbg_log_9b164efbe6db702f,
		"__wbg_warn_8342bfbc6028193a": WEBPACK_IMPORTED_MODULE_0.__wbg_warn_8342bfbc6028193a,
		"__wbindgen_string_get": WEBPACK_IMPORTED_MODULE_0.__wbindgen_string_get,
		"__wbg_parse_76a8a18ca3f8730b": WEBPACK_IMPORTED_MODULE_0.__wbg_parse_76a8a18ca3f8730b,
		"__wbg_stringify_d06ad2addc54d51e": WEBPACK_IMPORTED_MODULE_0.__wbg_stringify_d06ad2addc54d51e,
		"__wbg_set_07da13cc24b69217": WEBPACK_IMPORTED_MODULE_0.__wbg_set_07da13cc24b69217,
		"__wbindgen_debug_string": WEBPACK_IMPORTED_MODULE_0.__wbindgen_debug_string,
		"__wbindgen_throw": WEBPACK_IMPORTED_MODULE_0.__wbindgen_throw
	}
});

/***/ })

/******/ 	});
/************************************************************************/
/******/ 	// The module cache
/******/ 	var __webpack_module_cache__ = {};
/******/ 	
/******/ 	// The require function
/******/ 	function __webpack_require__(moduleId) {
/******/ 		// Check if module is in cache
/******/ 		var cachedModule = __webpack_module_cache__[moduleId];
/******/ 		if (cachedModule !== undefined) {
/******/ 			return cachedModule.exports;
/******/ 		}
/******/ 		// Create a new module (and put it into the cache)
/******/ 		var module = __webpack_module_cache__[moduleId] = {
/******/ 			id: moduleId,
/******/ 			loaded: false,
/******/ 			exports: {}
/******/ 		};
/******/ 	
/******/ 		// Execute the module function
/******/ 		__webpack_modules__[moduleId].call(module.exports, module, module.exports, __webpack_require__);
/******/ 	
/******/ 		// Flag the module as loaded
/******/ 		module.loaded = true;
/******/ 	
/******/ 		// Return the exports of the module
/******/ 		return module.exports;
/******/ 	}
/******/ 	
/******/ 	// expose the modules object (__webpack_modules__)
/******/ 	__webpack_require__.m = __webpack_modules__;
/******/ 	
/************************************************************************/
/******/ 	/* webpack/runtime/async module */
/******/ 	(() => {
/******/ 		var webpackQueues = typeof Symbol === "function" ? Symbol("webpack queues") : "__webpack_queues__";
/******/ 		var webpackExports = typeof Symbol === "function" ? Symbol("webpack exports") : "__webpack_exports__";
/******/ 		var webpackError = typeof Symbol === "function" ? Symbol("webpack error") : "__webpack_error__";
/******/ 		var resolveQueue = (queue) => {
/******/ 			if(queue && queue.d < 1) {
/******/ 				queue.d = 1;
/******/ 				queue.forEach((fn) => (fn.r--));
/******/ 				queue.forEach((fn) => (fn.r-- ? fn.r++ : fn()));
/******/ 			}
/******/ 		}
/******/ 		var wrapDeps = (deps) => (deps.map((dep) => {
/******/ 			if(dep !== null && typeof dep === "object") {
/******/ 				if(dep[webpackQueues]) return dep;
/******/ 				if(dep.then) {
/******/ 					var queue = [];
/******/ 					queue.d = 0;
/******/ 					dep.then((r) => {
/******/ 						obj[webpackExports] = r;
/******/ 						resolveQueue(queue);
/******/ 					}, (e) => {
/******/ 						obj[webpackError] = e;
/******/ 						resolveQueue(queue);
/******/ 					});
/******/ 					var obj = {};
/******/ 					obj[webpackQueues] = (fn) => (fn(queue));
/******/ 					return obj;
/******/ 				}
/******/ 			}
/******/ 			var ret = {};
/******/ 			ret[webpackQueues] = x => {};
/******/ 			ret[webpackExports] = dep;
/******/ 			return ret;
/******/ 		}));
/******/ 		__webpack_require__.a = (module, body, hasAwait) => {
/******/ 			var queue;
/******/ 			hasAwait && ((queue = []).d = -1);
/******/ 			var depQueues = new Set();
/******/ 			var exports = module.exports;
/******/ 			var currentDeps;
/******/ 			var outerResolve;
/******/ 			var reject;
/******/ 			var promise = new Promise((resolve, rej) => {
/******/ 				reject = rej;
/******/ 				outerResolve = resolve;
/******/ 			});
/******/ 			promise[webpackExports] = exports;
/******/ 			promise[webpackQueues] = (fn) => (queue && fn(queue), depQueues.forEach(fn), promise["catch"](x => {}));
/******/ 			module.exports = promise;
/******/ 			body((deps) => {
/******/ 				currentDeps = wrapDeps(deps);
/******/ 				var fn;
/******/ 				var getResult = () => (currentDeps.map((d) => {
/******/ 					if(d[webpackError]) throw d[webpackError];
/******/ 					return d[webpackExports];
/******/ 				}))
/******/ 				var promise = new Promise((resolve) => {
/******/ 					fn = () => (resolve(getResult));
/******/ 					fn.r = 0;
/******/ 					var fnQueue = (q) => (q !== queue && !depQueues.has(q) && (depQueues.add(q), q && !q.d && (fn.r++, q.push(fn))));
/******/ 					currentDeps.map((dep) => (dep[webpackQueues](fnQueue)));
/******/ 				});
/******/ 				return fn.r ? promise : getResult();
/******/ 			}, (err) => ((err ? reject(promise[webpackError] = err) : outerResolve(exports)), resolveQueue(queue)));
/******/ 			queue && queue.d < 0 && (queue.d = 0);
/******/ 		};
/******/ 	})();
/******/ 	
/******/ 	/* webpack/runtime/compat get default export */
/******/ 	(() => {
/******/ 		// getDefaultExport function for compatibility with non-harmony modules
/******/ 		__webpack_require__.n = (module) => {
/******/ 			var getter = module && module.__esModule ?
/******/ 				() => (module['default']) :
/******/ 				() => (module);
/******/ 			__webpack_require__.d(getter, { a: getter });
/******/ 			return getter;
/******/ 		};
/******/ 	})();
/******/ 	
/******/ 	/* webpack/runtime/define property getters */
/******/ 	(() => {
/******/ 		// define getter functions for harmony exports
/******/ 		__webpack_require__.d = (exports, definition) => {
/******/ 			for(var key in definition) {
/******/ 				if(__webpack_require__.o(definition, key) && !__webpack_require__.o(exports, key)) {
/******/ 					Object.defineProperty(exports, key, { enumerable: true, get: definition[key] });
/******/ 				}
/******/ 			}
/******/ 		};
/******/ 	})();
/******/ 	
/******/ 	/* webpack/runtime/ensure chunk */
/******/ 	(() => {
/******/ 		__webpack_require__.f = {};
/******/ 		// This file contains only the entry chunk.
/******/ 		// The chunk loading function for additional chunks
/******/ 		__webpack_require__.e = (chunkId) => {
/******/ 			return Promise.all(Object.keys(__webpack_require__.f).reduce((promises, key) => {
/******/ 				__webpack_require__.f[key](chunkId, promises);
/******/ 				return promises;
/******/ 			}, []));
/******/ 		};
/******/ 	})();
/******/ 	
/******/ 	/* webpack/runtime/get javascript chunk filename */
/******/ 	(() => {
/******/ 		// This function allow to reference async chunks
/******/ 		__webpack_require__.u = (chunkId) => {
/******/ 			// return url for filenames based on template
/******/ 			return "" + chunkId + ".js";
/******/ 		};
/******/ 	})();
/******/ 	
/******/ 	/* webpack/runtime/harmony module decorator */
/******/ 	(() => {
/******/ 		__webpack_require__.hmd = (module) => {
/******/ 			module = Object.create(module);
/******/ 			if (!module.children) module.children = [];
/******/ 			Object.defineProperty(module, 'exports', {
/******/ 				enumerable: true,
/******/ 				set: () => {
/******/ 					throw new Error('ES Modules may not assign module.exports or exports.*, Use ESM export syntax, instead: ' + module.id);
/******/ 				}
/******/ 			});
/******/ 			return module;
/******/ 		};
/******/ 	})();
/******/ 	
/******/ 	/* webpack/runtime/hasOwnProperty shorthand */
/******/ 	(() => {
/******/ 		__webpack_require__.o = (obj, prop) => (Object.prototype.hasOwnProperty.call(obj, prop))
/******/ 	})();
/******/ 	
/******/ 	/* webpack/runtime/make namespace object */
/******/ 	(() => {
/******/ 		// define __esModule on exports
/******/ 		__webpack_require__.r = (exports) => {
/******/ 			if(typeof Symbol !== 'undefined' && Symbol.toStringTag) {
/******/ 				Object.defineProperty(exports, Symbol.toStringTag, { value: 'Module' });
/******/ 			}
/******/ 			Object.defineProperty(exports, '__esModule', { value: true });
/******/ 		};
/******/ 	})();
/******/ 	
/******/ 	/* webpack/runtime/wasm loading */
/******/ 	(() => {
/******/ 		__webpack_require__.v = (exports, wasmModuleId, wasmModuleHash, importsObj) => {
/******/ 			var req = new Promise(function (resolve, reject) {
/******/ 				try {
/******/ 					var { readFile } = require('fs');
/******/ 					var { join } = require('path');
/******/ 		
/******/ 					readFile(join(__dirname, "" + wasmModuleHash + ".module.wasm"), function(err, buffer){
/******/ 						if (err) return reject(err);
/******/ 		
/******/ 						// Fake fetch response
/******/ 						resolve({
/******/ 							arrayBuffer() { return buffer; }
/******/ 						});
/******/ 					});
/******/ 				} catch (err) { reject(err); }
/******/ 			});
/******/ 			// no support for streaming compilation
/******/ 			return req
/******/ 				.then((x) => (x.arrayBuffer()))
/******/ 				.then((bytes) => (WebAssembly.instantiate(bytes, importsObj)))
/******/ 				.then((res) => (Object.assign(exports, res.instance.exports)));
/******/ 		};
/******/ 	})();
/******/ 	
/******/ 	/* webpack/runtime/publicPath */
/******/ 	(() => {
/******/ 		__webpack_require__.p = "";
/******/ 	})();
/******/ 	
/******/ 	/* webpack/runtime/require chunk loading */
/******/ 	(() => {
/******/ 		// no baseURI
/******/ 		
/******/ 		// object to store loaded chunks
/******/ 		// "1" means "loaded", otherwise not loaded yet
/******/ 		var installedChunks = {
/******/ 			"main": 1
/******/ 		};
/******/ 		
/******/ 		// no on chunks loaded
/******/ 		
/******/ 		var installChunk = (chunk) => {
/******/ 			var moreModules = chunk.modules, chunkIds = chunk.ids, runtime = chunk.runtime;
/******/ 			for(var moduleId in moreModules) {
/******/ 				if(__webpack_require__.o(moreModules, moduleId)) {
/******/ 					__webpack_require__.m[moduleId] = moreModules[moduleId];
/******/ 				}
/******/ 			}
/******/ 			if(runtime) runtime(__webpack_require__);
/******/ 			for(var i = 0; i < chunkIds.length; i++)
/******/ 				installedChunks[chunkIds[i]] = 1;
/******/ 		
/******/ 		};
/******/ 		
/******/ 		// require() chunk loading for javascript
/******/ 		__webpack_require__.f.require = (chunkId, promises) => {
/******/ 			// "1" is the signal for "already loaded"
/******/ 			if(!installedChunks[chunkId]) {
/******/ 				if(true) { // all chunks have JS
/******/ 					installChunk(require("./" + __webpack_require__.u(chunkId)));
/******/ 				} else installedChunks[chunkId] = 1;
/******/ 			}
/******/ 		};
/******/ 		
/******/ 		// no external install chunk
/******/ 		
/******/ 		// no HMR
/******/ 		
/******/ 		// no HMR manifest
/******/ 	})();
/******/ 	
/************************************************************************/
var __webpack_exports__ = {};
// This entry need to be wrapped in an IIFE because it need to be in strict mode.
(() => {
"use strict";
var exports = __webpack_exports__;
/*!*********************************!*\
  !*** ./noir_wasm/src/index.cts ***!
  \*********************************/


Object.defineProperty(exports, "__esModule", ({
  value: true
}));
exports.compile = compile;
var _path = __webpack_require__(/*! path */ "path");
var _noirWasmCompiler = __webpack_require__(/*! ./noir/noir-wasm-compiler */ "./noir_wasm/src/noir/noir-wasm-compiler.ts");
function _typeof(o) { "@babel/helpers - typeof"; return _typeof = "function" == typeof Symbol && "symbol" == typeof Symbol.iterator ? function (o) { return typeof o; } : function (o) { return o && "function" == typeof Symbol && o.constructor === Symbol && o !== Symbol.prototype ? "symbol" : typeof o; }, _typeof(o); }
function _regeneratorRuntime() { "use strict"; /*! regenerator-runtime -- Copyright (c) 2014-present, Facebook, Inc. -- license (MIT): https://github.com/facebook/regenerator/blob/main/LICENSE */ _regeneratorRuntime = function _regeneratorRuntime() { return e; }; var t, e = {}, r = Object.prototype, n = r.hasOwnProperty, o = Object.defineProperty || function (t, e, r) { t[e] = r.value; }, i = "function" == typeof Symbol ? Symbol : {}, a = i.iterator || "@@iterator", c = i.asyncIterator || "@@asyncIterator", u = i.toStringTag || "@@toStringTag"; function define(t, e, r) { return Object.defineProperty(t, e, { value: r, enumerable: !0, configurable: !0, writable: !0 }), t[e]; } try { define({}, ""); } catch (t) { define = function define(t, e, r) { return t[e] = r; }; } function wrap(t, e, r, n) { var i = e && e.prototype instanceof Generator ? e : Generator, a = Object.create(i.prototype), c = new Context(n || []); return o(a, "_invoke", { value: makeInvokeMethod(t, r, c) }), a; } function tryCatch(t, e, r) { try { return { type: "normal", arg: t.call(e, r) }; } catch (t) { return { type: "throw", arg: t }; } } e.wrap = wrap; var h = "suspendedStart", l = "suspendedYield", f = "executing", s = "completed", y = {}; function Generator() {} function GeneratorFunction() {} function GeneratorFunctionPrototype() {} var p = {}; define(p, a, function () { return this; }); var d = Object.getPrototypeOf, v = d && d(d(values([]))); v && v !== r && n.call(v, a) && (p = v); var g = GeneratorFunctionPrototype.prototype = Generator.prototype = Object.create(p); function defineIteratorMethods(t) { ["next", "throw", "return"].forEach(function (e) { define(t, e, function (t) { return this._invoke(e, t); }); }); } function AsyncIterator(t, e) { function invoke(r, o, i, a) { var c = tryCatch(t[r], t, o); if ("throw" !== c.type) { var u = c.arg, h = u.value; return h && "object" == _typeof(h) && n.call(h, "__await") ? e.resolve(h.__await).then(function (t) { invoke("next", t, i, a); }, function (t) { invoke("throw", t, i, a); }) : e.resolve(h).then(function (t) { u.value = t, i(u); }, function (t) { return invoke("throw", t, i, a); }); } a(c.arg); } var r; o(this, "_invoke", { value: function value(t, n) { function callInvokeWithMethodAndArg() { return new e(function (e, r) { invoke(t, n, e, r); }); } return r = r ? r.then(callInvokeWithMethodAndArg, callInvokeWithMethodAndArg) : callInvokeWithMethodAndArg(); } }); } function makeInvokeMethod(e, r, n) { var o = h; return function (i, a) { if (o === f) throw new Error("Generator is already running"); if (o === s) { if ("throw" === i) throw a; return { value: t, done: !0 }; } for (n.method = i, n.arg = a;;) { var c = n.delegate; if (c) { var u = maybeInvokeDelegate(c, n); if (u) { if (u === y) continue; return u; } } if ("next" === n.method) n.sent = n._sent = n.arg;else if ("throw" === n.method) { if (o === h) throw o = s, n.arg; n.dispatchException(n.arg); } else "return" === n.method && n.abrupt("return", n.arg); o = f; var p = tryCatch(e, r, n); if ("normal" === p.type) { if (o = n.done ? s : l, p.arg === y) continue; return { value: p.arg, done: n.done }; } "throw" === p.type && (o = s, n.method = "throw", n.arg = p.arg); } }; } function maybeInvokeDelegate(e, r) { var n = r.method, o = e.iterator[n]; if (o === t) return r.delegate = null, "throw" === n && e.iterator["return"] && (r.method = "return", r.arg = t, maybeInvokeDelegate(e, r), "throw" === r.method) || "return" !== n && (r.method = "throw", r.arg = new TypeError("The iterator does not provide a '" + n + "' method")), y; var i = tryCatch(o, e.iterator, r.arg); if ("throw" === i.type) return r.method = "throw", r.arg = i.arg, r.delegate = null, y; var a = i.arg; return a ? a.done ? (r[e.resultName] = a.value, r.next = e.nextLoc, "return" !== r.method && (r.method = "next", r.arg = t), r.delegate = null, y) : a : (r.method = "throw", r.arg = new TypeError("iterator result is not an object"), r.delegate = null, y); } function pushTryEntry(t) { var e = { tryLoc: t[0] }; 1 in t && (e.catchLoc = t[1]), 2 in t && (e.finallyLoc = t[2], e.afterLoc = t[3]), this.tryEntries.push(e); } function resetTryEntry(t) { var e = t.completion || {}; e.type = "normal", delete e.arg, t.completion = e; } function Context(t) { this.tryEntries = [{ tryLoc: "root" }], t.forEach(pushTryEntry, this), this.reset(!0); } function values(e) { if (e || "" === e) { var r = e[a]; if (r) return r.call(e); if ("function" == typeof e.next) return e; if (!isNaN(e.length)) { var o = -1, i = function next() { for (; ++o < e.length;) if (n.call(e, o)) return next.value = e[o], next.done = !1, next; return next.value = t, next.done = !0, next; }; return i.next = i; } } throw new TypeError(_typeof(e) + " is not iterable"); } return GeneratorFunction.prototype = GeneratorFunctionPrototype, o(g, "constructor", { value: GeneratorFunctionPrototype, configurable: !0 }), o(GeneratorFunctionPrototype, "constructor", { value: GeneratorFunction, configurable: !0 }), GeneratorFunction.displayName = define(GeneratorFunctionPrototype, u, "GeneratorFunction"), e.isGeneratorFunction = function (t) { var e = "function" == typeof t && t.constructor; return !!e && (e === GeneratorFunction || "GeneratorFunction" === (e.displayName || e.name)); }, e.mark = function (t) { return Object.setPrototypeOf ? Object.setPrototypeOf(t, GeneratorFunctionPrototype) : (t.__proto__ = GeneratorFunctionPrototype, define(t, u, "GeneratorFunction")), t.prototype = Object.create(g), t; }, e.awrap = function (t) { return { __await: t }; }, defineIteratorMethods(AsyncIterator.prototype), define(AsyncIterator.prototype, c, function () { return this; }), e.AsyncIterator = AsyncIterator, e.async = function (t, r, n, o, i) { void 0 === i && (i = Promise); var a = new AsyncIterator(wrap(t, r, n, o), i); return e.isGeneratorFunction(r) ? a : a.next().then(function (t) { return t.done ? t.value : a.next(); }); }, defineIteratorMethods(g), define(g, u, "Generator"), define(g, a, function () { return this; }), define(g, "toString", function () { return "[object Generator]"; }), e.keys = function (t) { var e = Object(t), r = []; for (var n in e) r.push(n); return r.reverse(), function next() { for (; r.length;) { var t = r.pop(); if (t in e) return next.value = t, next.done = !1, next; } return next.done = !0, next; }; }, e.values = values, Context.prototype = { constructor: Context, reset: function reset(e) { if (this.prev = 0, this.next = 0, this.sent = this._sent = t, this.done = !1, this.delegate = null, this.method = "next", this.arg = t, this.tryEntries.forEach(resetTryEntry), !e) for (var r in this) "t" === r.charAt(0) && n.call(this, r) && !isNaN(+r.slice(1)) && (this[r] = t); }, stop: function stop() { this.done = !0; var t = this.tryEntries[0].completion; if ("throw" === t.type) throw t.arg; return this.rval; }, dispatchException: function dispatchException(e) { if (this.done) throw e; var r = this; function handle(n, o) { return a.type = "throw", a.arg = e, r.next = n, o && (r.method = "next", r.arg = t), !!o; } for (var o = this.tryEntries.length - 1; o >= 0; --o) { var i = this.tryEntries[o], a = i.completion; if ("root" === i.tryLoc) return handle("end"); if (i.tryLoc <= this.prev) { var c = n.call(i, "catchLoc"), u = n.call(i, "finallyLoc"); if (c && u) { if (this.prev < i.catchLoc) return handle(i.catchLoc, !0); if (this.prev < i.finallyLoc) return handle(i.finallyLoc); } else if (c) { if (this.prev < i.catchLoc) return handle(i.catchLoc, !0); } else { if (!u) throw new Error("try statement without catch or finally"); if (this.prev < i.finallyLoc) return handle(i.finallyLoc); } } } }, abrupt: function abrupt(t, e) { for (var r = this.tryEntries.length - 1; r >= 0; --r) { var o = this.tryEntries[r]; if (o.tryLoc <= this.prev && n.call(o, "finallyLoc") && this.prev < o.finallyLoc) { var i = o; break; } } i && ("break" === t || "continue" === t) && i.tryLoc <= e && e <= i.finallyLoc && (i = null); var a = i ? i.completion : {}; return a.type = t, a.arg = e, i ? (this.method = "next", this.next = i.finallyLoc, y) : this.complete(a); }, complete: function complete(t, e) { if ("throw" === t.type) throw t.arg; return "break" === t.type || "continue" === t.type ? this.next = t.arg : "return" === t.type ? (this.rval = this.arg = t.arg, this.method = "return", this.next = "end") : "normal" === t.type && e && (this.next = e), y; }, finish: function finish(t) { for (var e = this.tryEntries.length - 1; e >= 0; --e) { var r = this.tryEntries[e]; if (r.finallyLoc === t) return this.complete(r.completion, r.afterLoc), resetTryEntry(r), y; } }, "catch": function _catch(t) { for (var e = this.tryEntries.length - 1; e >= 0; --e) { var r = this.tryEntries[e]; if (r.tryLoc === t) { var n = r.completion; if ("throw" === n.type) { var o = n.arg; resetTryEntry(r); } return o; } } throw new Error("illegal catch attempt"); }, delegateYield: function delegateYield(e, r, n) { return this.delegate = { iterator: values(e), resultName: r, nextLoc: n }, "next" === this.method && (this.arg = t), y; } }, e; }
function asyncGeneratorStep(gen, resolve, reject, _next, _throw, key, arg) { try { var info = gen[key](arg); var value = info.value; } catch (error) { reject(error); return; } if (info.done) { resolve(value); } else { Promise.resolve(value).then(_next, _throw); } }
function _asyncToGenerator(fn) { return function () { var self = this, args = arguments; return new Promise(function (resolve, reject) { var gen = fn.apply(self, args); function _next(value) { asyncGeneratorStep(gen, resolve, reject, _next, _throw, "next", value); } function _throw(err) { asyncGeneratorStep(gen, resolve, reject, _next, _throw, "throw", err); } _next(undefined); }); }; }
/**
 * Given a Nargo output generates an Aztec-compatible contract artifact.
 * @param compiled - Noir build output.
 * @returns Aztec contract build artifact.
 */
function generateProgramArtifact(_ref, noir_version) {
  var program = _ref.program;
  return {
    noir_version: noir_version,
    hash: program.hash,
    backend: program.backend,
    abi: program.abi
  };
}
function compileUsingNoirWasm(_x, _x2) {
  return _compileUsingNoirWasm.apply(this, arguments);
}
function _compileUsingNoirWasm() {
  _compileUsingNoirWasm = _asyncToGenerator( /*#__PURE__*/_regeneratorRuntime().mark(function _callee(projectPath, wasmCompiler) {
    var compiler, artifacts;
    return _regeneratorRuntime().wrap(function _callee$(_context) {
      while (1) switch (_context.prev = _context.next) {
        case 0:
          compiler = _noirWasmCompiler.NoirWasmContractCompiler["new"]((0, _path.resolve)(projectPath), wasmCompiler);
          _context.next = 3;
          return compiler.compile();
        case 3:
          artifacts = _context.sent;
          return _context.abrupt("return", artifacts.map(function (artifact) {
            return generateProgramArtifact(artifact);
          }));
        case 5:
        case "end":
          return _context.stop();
      }
    }, _callee);
  }));
  return _compileUsingNoirWasm.apply(this, arguments);
}
function compile(_x3) {
  return _compile.apply(this, arguments);
}
function _compile() {
  _compile = _asyncToGenerator( /*#__PURE__*/_regeneratorRuntime().mark(function _callee2(projectPath) {
    var cjsModule, esmModule;
    return _regeneratorRuntime().wrap(function _callee2$(_context2) {
      while (1) switch (_context2.prev = _context2.next) {
        case 0:
          if (false) {}
          _context2.next = 3;
          return __webpack_require__(/*! ../cjs */ "./noir_wasm/cjs/index.js");
        case 3:
          cjsModule = _context2.sent;
          return _context2.abrupt("return", cjsModule.compile(projectPath));
        case 7:
          _context2.next = 9;
          return __webpack_require__.e(/*! import() */ "noir_wasm_esm_index_js").then(__webpack_require__.bind(__webpack_require__, /*! ../esm */ "./noir_wasm/esm/index.js"));
        case 9:
          esmModule = _context2.sent;
          return _context2.abrupt("return", esmModule.compile(projectPath));
        case 11:
        case "end":
          return _context2.stop();
      }
    }, _callee2);
  }));
  return _compile.apply(this, arguments);
}
})();

module.exports = __webpack_exports__;
/******/ })()
;
//# sourceMappingURL=main.js.map