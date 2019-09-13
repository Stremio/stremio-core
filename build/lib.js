"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.load = load;

var _stremio_core_web = _interopRequireWildcard(require("./stremio_core_web.js"));

function _getRequireWildcardCache() { if (typeof WeakMap !== "function") return null; var cache = new WeakMap(); _getRequireWildcardCache = function _getRequireWildcardCache() { return cache; }; return cache; }

function _interopRequireWildcard(obj) { if (obj && obj.__esModule) { return obj; } var cache = _getRequireWildcardCache(); if (cache && cache.has(obj)) { return cache.get(obj); } var newObj = {}; if (obj != null) { var hasPropertyDescriptor = Object.defineProperty && Object.getOwnPropertyDescriptor; for (var key in obj) { if (Object.prototype.hasOwnProperty.call(obj, key)) { var desc = hasPropertyDescriptor ? Object.getOwnPropertyDescriptor(obj, key) : null; if (desc && (desc.get || desc.set)) { Object.defineProperty(newObj, key, desc); } else { newObj[key] = obj[key]; } } } } newObj["default"] = obj; if (cache) { cache.set(obj, newObj); } return newObj; }

function load() {
  return (0, _stremio_core_web["default"])('stremio_core_web.wasm').then(function () {
    var listeners = {};
    var containerService = new _stremio_core_web.ContainerService(function (event) {
      if (Array.isArray(listeners[event.name])) {
        listeners[event.name].forEach(function (listener) {
          listener(event.args);
        });
      }
    });
    window.stateContainer = Object.freeze({
      on: function on(eventName, listener) {
        listeners[eventName] = listeners[eventName] || [];

        if (listeners[eventName].indexOf(listener) === -1) {
          listeners[eventName].push(listener);
        }
      },
      off: function off(eventName, listener) {
        if (Array.isArray(listeners[eventName])) {
          var listenerIndex = listeners[eventName].indexOf(listener);

          if (listenerIndex !== -1) {
            listeners[eventName].splice(listenerIndex, 1);
          }
        }
      },
      dispatch: function dispatch(_ref) {
        var action = _ref.action,
            args = _ref.args;
        containerService.dispatch({
          action: action,
          args: args
        });
      },
      getState: function getState() {
        return containerService.get_state();
      }
    });
  });
}
