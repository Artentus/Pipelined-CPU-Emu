/******/ (function(modules) { // webpackBootstrap
/******/ 	// install a JSONP callback for chunk loading
/******/ 	function webpackJsonpCallback(data) {
/******/ 		var chunkIds = data[0];
/******/ 		var moreModules = data[1];
/******/
/******/
/******/ 		// add "moreModules" to the modules object,
/******/ 		// then flag all "chunkIds" as loaded and fire callback
/******/ 		var moduleId, chunkId, i = 0, resolves = [];
/******/ 		for(;i < chunkIds.length; i++) {
/******/ 			chunkId = chunkIds[i];
/******/ 			if(Object.prototype.hasOwnProperty.call(installedChunks, chunkId) && installedChunks[chunkId]) {
/******/ 				resolves.push(installedChunks[chunkId][0]);
/******/ 			}
/******/ 			installedChunks[chunkId] = 0;
/******/ 		}
/******/ 		for(moduleId in moreModules) {
/******/ 			if(Object.prototype.hasOwnProperty.call(moreModules, moduleId)) {
/******/ 				modules[moduleId] = moreModules[moduleId];
/******/ 			}
/******/ 		}
/******/ 		if(parentJsonpFunction) parentJsonpFunction(data);
/******/
/******/ 		while(resolves.length) {
/******/ 			resolves.shift()();
/******/ 		}
/******/
/******/ 	};
/******/
/******/
/******/ 	// The module cache
/******/ 	var installedModules = {};
/******/
/******/ 	// object to store loaded and loading chunks
/******/ 	// undefined = chunk not loaded, null = chunk preloaded/prefetched
/******/ 	// Promise = chunk loading, 0 = chunk loaded
/******/ 	var installedChunks = {
/******/ 		"main": 0
/******/ 	};
/******/
/******/
/******/
/******/ 	// script path function
/******/ 	function jsonpScriptSrc(chunkId) {
/******/ 		return __webpack_require__.p + "" + chunkId + ".bootstrap.js"
/******/ 	}
/******/
/******/ 	// object to store loaded and loading wasm modules
/******/ 	var installedWasmModules = {};
/******/
/******/ 	function promiseResolve() { return Promise.resolve(); }
/******/
/******/ 	var wasmImportObjects = {
/******/ 		"./node_modules/@artentus/jam1emu/jam1emu_lib_bg.wasm": function() {
/******/ 			return {
/******/ 				"./jam1emu_lib_bg.js": {
/******/ 					"__wbg_print_a667856ee1bc7583": function(p0i32,p1i32) {
/******/ 						return installedModules["./node_modules/@artentus/jam1emu/jam1emu_lib_bg.js"].exports["__wbg_print_a667856ee1bc7583"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_readuartdata_7fe93d71a283c596": function(p0i32) {
/******/ 						return installedModules["./node_modules/@artentus/jam1emu/jam1emu_lib_bg.js"].exports["__wbg_readuartdata_7fe93d71a283c596"](p0i32);
/******/ 					},
/******/ 					"__wbindgen_cb_drop": function(p0i32) {
/******/ 						return installedModules["./node_modules/@artentus/jam1emu/jam1emu_lib_bg.js"].exports["__wbindgen_cb_drop"](p0i32);
/******/ 					},
/******/ 					"__wbindgen_object_drop_ref": function(p0i32) {
/******/ 						return installedModules["./node_modules/@artentus/jam1emu/jam1emu_lib_bg.js"].exports["__wbindgen_object_drop_ref"](p0i32);
/******/ 					},
/******/ 					"__wbindgen_number_get": function(p0i32,p1i32) {
/******/ 						return installedModules["./node_modules/@artentus/jam1emu/jam1emu_lib_bg.js"].exports["__wbindgen_number_get"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbindgen_is_null": function(p0i32) {
/******/ 						return installedModules["./node_modules/@artentus/jam1emu/jam1emu_lib_bg.js"].exports["__wbindgen_is_null"](p0i32);
/******/ 					},
/******/ 					"__wbindgen_boolean_get": function(p0i32) {
/******/ 						return installedModules["./node_modules/@artentus/jam1emu/jam1emu_lib_bg.js"].exports["__wbindgen_boolean_get"](p0i32);
/******/ 					},
/******/ 					"__wbg_instanceof_Window_acc97ff9f5d2c7b4": function(p0i32) {
/******/ 						return installedModules["./node_modules/@artentus/jam1emu/jam1emu_lib_bg.js"].exports["__wbg_instanceof_Window_acc97ff9f5d2c7b4"](p0i32);
/******/ 					},
/******/ 					"__wbg_navigator_d1dcf282b97e2495": function(p0i32) {
/******/ 						return installedModules["./node_modules/@artentus/jam1emu/jam1emu_lib_bg.js"].exports["__wbg_navigator_d1dcf282b97e2495"](p0i32);
/******/ 					},
/******/ 					"__wbg_isSecureContext_541f52c311a1c679": function(p0i32) {
/******/ 						return installedModules["./node_modules/@artentus/jam1emu/jam1emu_lib_bg.js"].exports["__wbg_isSecureContext_541f52c311a1c679"](p0i32);
/******/ 					},
/******/ 					"__wbg_setTimeout_d6fcf0d9067b8e64": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["./node_modules/@artentus/jam1emu/jam1emu_lib_bg.js"].exports["__wbg_setTimeout_d6fcf0d9067b8e64"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbg_getGamepads_8001a499f2b689fe": function(p0i32) {
/******/ 						return installedModules["./node_modules/@artentus/jam1emu/jam1emu_lib_bg.js"].exports["__wbg_getGamepads_8001a499f2b689fe"](p0i32);
/******/ 					},
/******/ 					"__wbg_id_55b63ccda43785eb": function(p0i32,p1i32) {
/******/ 						return installedModules["./node_modules/@artentus/jam1emu/jam1emu_lib_bg.js"].exports["__wbg_id_55b63ccda43785eb"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_index_94744625261e9824": function(p0i32) {
/******/ 						return installedModules["./node_modules/@artentus/jam1emu/jam1emu_lib_bg.js"].exports["__wbg_index_94744625261e9824"](p0i32);
/******/ 					},
/******/ 					"__wbg_mapping_778a451256a4d95e": function(p0i32) {
/******/ 						return installedModules["./node_modules/@artentus/jam1emu/jam1emu_lib_bg.js"].exports["__wbg_mapping_778a451256a4d95e"](p0i32);
/******/ 					},
/******/ 					"__wbindgen_string_get": function(p0i32,p1i32) {
/******/ 						return installedModules["./node_modules/@artentus/jam1emu/jam1emu_lib_bg.js"].exports["__wbindgen_string_get"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_connected_41b85c162970593b": function(p0i32) {
/******/ 						return installedModules["./node_modules/@artentus/jam1emu/jam1emu_lib_bg.js"].exports["__wbg_connected_41b85c162970593b"](p0i32);
/******/ 					},
/******/ 					"__wbg_buttons_1162e62c0dc4246e": function(p0i32) {
/******/ 						return installedModules["./node_modules/@artentus/jam1emu/jam1emu_lib_bg.js"].exports["__wbg_buttons_1162e62c0dc4246e"](p0i32);
/******/ 					},
/******/ 					"__wbg_axes_385390941534cfd7": function(p0i32) {
/******/ 						return installedModules["./node_modules/@artentus/jam1emu/jam1emu_lib_bg.js"].exports["__wbg_axes_385390941534cfd7"](p0i32);
/******/ 					},
/******/ 					"__wbg_pressed_7add67434a3dd765": function(p0i32) {
/******/ 						return installedModules["./node_modules/@artentus/jam1emu/jam1emu_lib_bg.js"].exports["__wbg_pressed_7add67434a3dd765"](p0i32);
/******/ 					},
/******/ 					"__wbindgen_string_new": function(p0i32,p1i32) {
/******/ 						return installedModules["./node_modules/@artentus/jam1emu/jam1emu_lib_bg.js"].exports["__wbindgen_string_new"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbindgen_number_new": function(p0f64) {
/******/ 						return installedModules["./node_modules/@artentus/jam1emu/jam1emu_lib_bg.js"].exports["__wbindgen_number_new"](p0f64);
/******/ 					},
/******/ 					"__wbg_destination_df4e9893e562390a": function(p0i32) {
/******/ 						return installedModules["./node_modules/@artentus/jam1emu/jam1emu_lib_bg.js"].exports["__wbg_destination_df4e9893e562390a"](p0i32);
/******/ 					},
/******/ 					"__wbg_currentTime_80316e838e7d1028": function(p0i32) {
/******/ 						return installedModules["./node_modules/@artentus/jam1emu/jam1emu_lib_bg.js"].exports["__wbg_currentTime_80316e838e7d1028"](p0i32);
/******/ 					},
/******/ 					"__wbg_newwithcontextoptions_0d1099da75124451": function(p0i32) {
/******/ 						return installedModules["./node_modules/@artentus/jam1emu/jam1emu_lib_bg.js"].exports["__wbg_newwithcontextoptions_0d1099da75124451"](p0i32);
/******/ 					},
/******/ 					"__wbg_close_7d5aa2babb9d8fc2": function(p0i32) {
/******/ 						return installedModules["./node_modules/@artentus/jam1emu/jam1emu_lib_bg.js"].exports["__wbg_close_7d5aa2babb9d8fc2"](p0i32);
/******/ 					},
/******/ 					"__wbg_createBuffer_47ef089f86b99318": function(p0i32,p1i32,p2i32,p3f32) {
/******/ 						return installedModules["./node_modules/@artentus/jam1emu/jam1emu_lib_bg.js"].exports["__wbg_createBuffer_47ef089f86b99318"](p0i32,p1i32,p2i32,p3f32);
/******/ 					},
/******/ 					"__wbg_createBufferSource_f06449934aee7f6f": function(p0i32) {
/******/ 						return installedModules["./node_modules/@artentus/jam1emu/jam1emu_lib_bg.js"].exports["__wbg_createBufferSource_f06449934aee7f6f"](p0i32);
/******/ 					},
/******/ 					"__wbg_resume_244684c4c6bb49fa": function(p0i32) {
/******/ 						return installedModules["./node_modules/@artentus/jam1emu/jam1emu_lib_bg.js"].exports["__wbg_resume_244684c4c6bb49fa"](p0i32);
/******/ 					},
/******/ 					"__wbg_setbuffer_ead89b52e0bf1c40": function(p0i32,p1i32) {
/******/ 						return installedModules["./node_modules/@artentus/jam1emu/jam1emu_lib_bg.js"].exports["__wbg_setbuffer_ead89b52e0bf1c40"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_setonended_ae460558754eae1e": function(p0i32,p1i32) {
/******/ 						return installedModules["./node_modules/@artentus/jam1emu/jam1emu_lib_bg.js"].exports["__wbg_setonended_ae460558754eae1e"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_start_e330425e284a693a": function(p0i32,p1f64) {
/******/ 						return installedModules["./node_modules/@artentus/jam1emu/jam1emu_lib_bg.js"].exports["__wbg_start_e330425e284a693a"](p0i32,p1f64);
/******/ 					},
/******/ 					"__wbg_connect_463d4300ff833991": function(p0i32,p1i32) {
/******/ 						return installedModules["./node_modules/@artentus/jam1emu/jam1emu_lib_bg.js"].exports["__wbg_connect_463d4300ff833991"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_instanceof_DomException_c2b4ae110dc047f8": function(p0i32) {
/******/ 						return installedModules["./node_modules/@artentus/jam1emu/jam1emu_lib_bg.js"].exports["__wbg_instanceof_DomException_c2b4ae110dc047f8"](p0i32);
/******/ 					},
/******/ 					"__wbg_message_a7af3ee0cc0fe28d": function(p0i32,p1i32) {
/******/ 						return installedModules["./node_modules/@artentus/jam1emu/jam1emu_lib_bg.js"].exports["__wbg_message_a7af3ee0cc0fe28d"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_copyToChannel_e683ef3e184292ab": function(p0i32,p1i32,p2i32,p3i32) {
/******/ 						return installedModules["./node_modules/@artentus/jam1emu/jam1emu_lib_bg.js"].exports["__wbg_copyToChannel_e683ef3e184292ab"](p0i32,p1i32,p2i32,p3i32);
/******/ 					},
/******/ 					"__wbg_get_57245cc7d7c7619d": function(p0i32,p1i32) {
/******/ 						return installedModules["./node_modules/@artentus/jam1emu/jam1emu_lib_bg.js"].exports["__wbg_get_57245cc7d7c7619d"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_length_6e3bbe7c8bd4dbd8": function(p0i32) {
/******/ 						return installedModules["./node_modules/@artentus/jam1emu/jam1emu_lib_bg.js"].exports["__wbg_length_6e3bbe7c8bd4dbd8"](p0i32);
/******/ 					},
/******/ 					"__wbg_newnoargs_b5b063fc6c2f0376": function(p0i32,p1i32) {
/******/ 						return installedModules["./node_modules/@artentus/jam1emu/jam1emu_lib_bg.js"].exports["__wbg_newnoargs_b5b063fc6c2f0376"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_call_97ae9d8645dc388b": function(p0i32,p1i32) {
/******/ 						return installedModules["./node_modules/@artentus/jam1emu/jam1emu_lib_bg.js"].exports["__wbg_call_97ae9d8645dc388b"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_new_0b9bfdd97583284e": function() {
/******/ 						return installedModules["./node_modules/@artentus/jam1emu/jam1emu_lib_bg.js"].exports["__wbg_new_0b9bfdd97583284e"]();
/******/ 					},
/******/ 					"__wbindgen_object_clone_ref": function(p0i32) {
/******/ 						return installedModules["./node_modules/@artentus/jam1emu/jam1emu_lib_bg.js"].exports["__wbindgen_object_clone_ref"](p0i32);
/******/ 					},
/******/ 					"__wbg_self_6d479506f72c6a71": function() {
/******/ 						return installedModules["./node_modules/@artentus/jam1emu/jam1emu_lib_bg.js"].exports["__wbg_self_6d479506f72c6a71"]();
/******/ 					},
/******/ 					"__wbg_window_f2557cc78490aceb": function() {
/******/ 						return installedModules["./node_modules/@artentus/jam1emu/jam1emu_lib_bg.js"].exports["__wbg_window_f2557cc78490aceb"]();
/******/ 					},
/******/ 					"__wbg_globalThis_7f206bda628d5286": function() {
/******/ 						return installedModules["./node_modules/@artentus/jam1emu/jam1emu_lib_bg.js"].exports["__wbg_globalThis_7f206bda628d5286"]();
/******/ 					},
/******/ 					"__wbg_global_ba75c50d1cf384f4": function() {
/******/ 						return installedModules["./node_modules/@artentus/jam1emu/jam1emu_lib_bg.js"].exports["__wbg_global_ba75c50d1cf384f4"]();
/******/ 					},
/******/ 					"__wbindgen_is_undefined": function(p0i32) {
/******/ 						return installedModules["./node_modules/@artentus/jam1emu/jam1emu_lib_bg.js"].exports["__wbindgen_is_undefined"](p0i32);
/******/ 					},
/******/ 					"__wbg_eval_6dc8993472839847": function(p0i32,p1i32) {
/******/ 						return installedModules["./node_modules/@artentus/jam1emu/jam1emu_lib_bg.js"].exports["__wbg_eval_6dc8993472839847"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_now_58886682b7e790d7": function() {
/******/ 						return installedModules["./node_modules/@artentus/jam1emu/jam1emu_lib_bg.js"].exports["__wbg_now_58886682b7e790d7"]();
/******/ 					},
/******/ 					"__wbg_set_bf3f89b92d5a34bf": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["./node_modules/@artentus/jam1emu/jam1emu_lib_bg.js"].exports["__wbg_set_bf3f89b92d5a34bf"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbindgen_debug_string": function(p0i32,p1i32) {
/******/ 						return installedModules["./node_modules/@artentus/jam1emu/jam1emu_lib_bg.js"].exports["__wbindgen_debug_string"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbindgen_throw": function(p0i32,p1i32) {
/******/ 						return installedModules["./node_modules/@artentus/jam1emu/jam1emu_lib_bg.js"].exports["__wbindgen_throw"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbindgen_closure_wrapper515": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["./node_modules/@artentus/jam1emu/jam1emu_lib_bg.js"].exports["__wbindgen_closure_wrapper515"](p0i32,p1i32,p2i32);
/******/ 					}
/******/ 				},
/******/ 				"./snippets/jam1emu-6a5a5a2c3f260b1a/terminal.js": {
/******/ 					"attach": function() {
/******/ 						return installedModules["./node_modules/@artentus/jam1emu/snippets/jam1emu-6a5a5a2c3f260b1a/terminal.js"].exports["attach"]();
/******/ 					}
/******/ 				}
/******/ 			};
/******/ 		},
/******/ 	};
/******/
/******/ 	// The require function
/******/ 	function __webpack_require__(moduleId) {
/******/
/******/ 		// Check if module is in cache
/******/ 		if(installedModules[moduleId]) {
/******/ 			return installedModules[moduleId].exports;
/******/ 		}
/******/ 		// Create a new module (and put it into the cache)
/******/ 		var module = installedModules[moduleId] = {
/******/ 			i: moduleId,
/******/ 			l: false,
/******/ 			exports: {}
/******/ 		};
/******/
/******/ 		// Execute the module function
/******/ 		modules[moduleId].call(module.exports, module, module.exports, __webpack_require__);
/******/
/******/ 		// Flag the module as loaded
/******/ 		module.l = true;
/******/
/******/ 		// Return the exports of the module
/******/ 		return module.exports;
/******/ 	}
/******/
/******/ 	// This file contains only the entry chunk.
/******/ 	// The chunk loading function for additional chunks
/******/ 	__webpack_require__.e = function requireEnsure(chunkId) {
/******/ 		var promises = [];
/******/
/******/
/******/ 		// JSONP chunk loading for javascript
/******/
/******/ 		var installedChunkData = installedChunks[chunkId];
/******/ 		if(installedChunkData !== 0) { // 0 means "already installed".
/******/
/******/ 			// a Promise means "currently loading".
/******/ 			if(installedChunkData) {
/******/ 				promises.push(installedChunkData[2]);
/******/ 			} else {
/******/ 				// setup Promise in chunk cache
/******/ 				var promise = new Promise(function(resolve, reject) {
/******/ 					installedChunkData = installedChunks[chunkId] = [resolve, reject];
/******/ 				});
/******/ 				promises.push(installedChunkData[2] = promise);
/******/
/******/ 				// start chunk loading
/******/ 				var script = document.createElement('script');
/******/ 				var onScriptComplete;
/******/
/******/ 				script.charset = 'utf-8';
/******/ 				script.timeout = 120;
/******/ 				if (__webpack_require__.nc) {
/******/ 					script.setAttribute("nonce", __webpack_require__.nc);
/******/ 				}
/******/ 				script.src = jsonpScriptSrc(chunkId);
/******/
/******/ 				// create error before stack unwound to get useful stacktrace later
/******/ 				var error = new Error();
/******/ 				onScriptComplete = function (event) {
/******/ 					// avoid mem leaks in IE.
/******/ 					script.onerror = script.onload = null;
/******/ 					clearTimeout(timeout);
/******/ 					var chunk = installedChunks[chunkId];
/******/ 					if(chunk !== 0) {
/******/ 						if(chunk) {
/******/ 							var errorType = event && (event.type === 'load' ? 'missing' : event.type);
/******/ 							var realSrc = event && event.target && event.target.src;
/******/ 							error.message = 'Loading chunk ' + chunkId + ' failed.\n(' + errorType + ': ' + realSrc + ')';
/******/ 							error.name = 'ChunkLoadError';
/******/ 							error.type = errorType;
/******/ 							error.request = realSrc;
/******/ 							chunk[1](error);
/******/ 						}
/******/ 						installedChunks[chunkId] = undefined;
/******/ 					}
/******/ 				};
/******/ 				var timeout = setTimeout(function(){
/******/ 					onScriptComplete({ type: 'timeout', target: script });
/******/ 				}, 120000);
/******/ 				script.onerror = script.onload = onScriptComplete;
/******/ 				document.head.appendChild(script);
/******/ 			}
/******/ 		}
/******/
/******/ 		// Fetch + compile chunk loading for webassembly
/******/
/******/ 		var wasmModules = {"0":["./node_modules/@artentus/jam1emu/jam1emu_lib_bg.wasm"]}[chunkId] || [];
/******/
/******/ 		wasmModules.forEach(function(wasmModuleId) {
/******/ 			var installedWasmModuleData = installedWasmModules[wasmModuleId];
/******/
/******/ 			// a Promise means "currently loading" or "already loaded".
/******/ 			if(installedWasmModuleData)
/******/ 				promises.push(installedWasmModuleData);
/******/ 			else {
/******/ 				var importObject = wasmImportObjects[wasmModuleId]();
/******/ 				var req = fetch(__webpack_require__.p + "" + {"./node_modules/@artentus/jam1emu/jam1emu_lib_bg.wasm":"81f824773ad564a491d6"}[wasmModuleId] + ".module.wasm");
/******/ 				var promise;
/******/ 				if(importObject instanceof Promise && typeof WebAssembly.compileStreaming === 'function') {
/******/ 					promise = Promise.all([WebAssembly.compileStreaming(req), importObject]).then(function(items) {
/******/ 						return WebAssembly.instantiate(items[0], items[1]);
/******/ 					});
/******/ 				} else if(typeof WebAssembly.instantiateStreaming === 'function') {
/******/ 					promise = WebAssembly.instantiateStreaming(req, importObject);
/******/ 				} else {
/******/ 					var bytesPromise = req.then(function(x) { return x.arrayBuffer(); });
/******/ 					promise = bytesPromise.then(function(bytes) {
/******/ 						return WebAssembly.instantiate(bytes, importObject);
/******/ 					});
/******/ 				}
/******/ 				promises.push(installedWasmModules[wasmModuleId] = promise.then(function(res) {
/******/ 					return __webpack_require__.w[wasmModuleId] = (res.instance || res).exports;
/******/ 				}));
/******/ 			}
/******/ 		});
/******/ 		return Promise.all(promises);
/******/ 	};
/******/
/******/ 	// expose the modules object (__webpack_modules__)
/******/ 	__webpack_require__.m = modules;
/******/
/******/ 	// expose the module cache
/******/ 	__webpack_require__.c = installedModules;
/******/
/******/ 	// define getter function for harmony exports
/******/ 	__webpack_require__.d = function(exports, name, getter) {
/******/ 		if(!__webpack_require__.o(exports, name)) {
/******/ 			Object.defineProperty(exports, name, { enumerable: true, get: getter });
/******/ 		}
/******/ 	};
/******/
/******/ 	// define __esModule on exports
/******/ 	__webpack_require__.r = function(exports) {
/******/ 		if(typeof Symbol !== 'undefined' && Symbol.toStringTag) {
/******/ 			Object.defineProperty(exports, Symbol.toStringTag, { value: 'Module' });
/******/ 		}
/******/ 		Object.defineProperty(exports, '__esModule', { value: true });
/******/ 	};
/******/
/******/ 	// create a fake namespace object
/******/ 	// mode & 1: value is a module id, require it
/******/ 	// mode & 2: merge all properties of value into the ns
/******/ 	// mode & 4: return value when already ns object
/******/ 	// mode & 8|1: behave like require
/******/ 	__webpack_require__.t = function(value, mode) {
/******/ 		if(mode & 1) value = __webpack_require__(value);
/******/ 		if(mode & 8) return value;
/******/ 		if((mode & 4) && typeof value === 'object' && value && value.__esModule) return value;
/******/ 		var ns = Object.create(null);
/******/ 		__webpack_require__.r(ns);
/******/ 		Object.defineProperty(ns, 'default', { enumerable: true, value: value });
/******/ 		if(mode & 2 && typeof value != 'string') for(var key in value) __webpack_require__.d(ns, key, function(key) { return value[key]; }.bind(null, key));
/******/ 		return ns;
/******/ 	};
/******/
/******/ 	// getDefaultExport function for compatibility with non-harmony modules
/******/ 	__webpack_require__.n = function(module) {
/******/ 		var getter = module && module.__esModule ?
/******/ 			function getDefault() { return module['default']; } :
/******/ 			function getModuleExports() { return module; };
/******/ 		__webpack_require__.d(getter, 'a', getter);
/******/ 		return getter;
/******/ 	};
/******/
/******/ 	// Object.prototype.hasOwnProperty.call
/******/ 	__webpack_require__.o = function(object, property) { return Object.prototype.hasOwnProperty.call(object, property); };
/******/
/******/ 	// __webpack_public_path__
/******/ 	__webpack_require__.p = "";
/******/
/******/ 	// on error function for async loading
/******/ 	__webpack_require__.oe = function(err) { console.error(err); throw err; };
/******/
/******/ 	// object with all WebAssembly.instance exports
/******/ 	__webpack_require__.w = {};
/******/
/******/ 	var jsonpArray = window["webpackJsonp"] = window["webpackJsonp"] || [];
/******/ 	var oldJsonpFunction = jsonpArray.push.bind(jsonpArray);
/******/ 	jsonpArray.push = webpackJsonpCallback;
/******/ 	jsonpArray = jsonpArray.slice();
/******/ 	for(var i = 0; i < jsonpArray.length; i++) webpackJsonpCallback(jsonpArray[i]);
/******/ 	var parentJsonpFunction = oldJsonpFunction;
/******/
/******/
/******/ 	// Load entry module and return exports
/******/ 	return __webpack_require__(__webpack_require__.s = "./bootstrap.js");
/******/ })
/************************************************************************/
/******/ ({

/***/ "./bootstrap.js":
/*!**********************!*\
  !*** ./bootstrap.js ***!
  \**********************/
/*! no static exports found */
/***/ (function(module, exports, __webpack_require__) {

eval("// A dependency graph that contains any wasm must all be imported\r\n// asynchronously. This `bootstrap.js` file does the single async import, so\r\n// that no one else needs to worry about it again.\r\nPromise.all(/*! import() */[__webpack_require__.e(0), __webpack_require__.e(1)]).then(__webpack_require__.bind(null, /*! ./index.js */ \"./index.js\"))\r\n  .catch(e => console.error(\"Error importing `index.js`:\", e));\r\n\n\n//# sourceURL=webpack:///./bootstrap.js?");

/***/ })

/******/ });