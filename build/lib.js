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
    var events = {};
    var containerService = new _stremio_core_web.ContainerService(function (_ref) {
      var action = _ref.action,
          args = _ref.args;

      if (Array.isArray(events[action])) {
        events[action].forEach(function (listener) {
          listener(args);
        });
      }
    });
    window.stateContainer = Object.freeze({
      on: function on(eventName, listener) {
        events[eventName] = events[eventName] || [];

        if (events[eventName].indexOf(listener) === -1) {
          events[eventName].push(listener);
        }
      },
      off: function off(eventName, listener) {
        if (Array.isArray(events[eventName])) {
          var listenerIndex = events[eventName].indexOf(listener);

          if (listenerIndex !== -1) {
            events[eventName].splice(listenerIndex, 1);
          }
        }
      },
      dispatch: function dispatch(_ref2) {
        var action = _ref2.action,
            args = _ref2.args;
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
