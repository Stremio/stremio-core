import compileStateContainer, { ContainerService } from './stremio_core_web.js';

export function load() {
    return compileStateContainer('stremio_core_web.wasm')
        .then(() => {
            const listeners = {};
            const containerService = new ContainerService((event) => {
                if (Array.isArray(listeners[event.name])) {
                    listeners[event.name].forEach((listener) => {
                        listener(event.args);
                    });
                }
            });

            window.stateContainer = Object.freeze({
                on: function(eventName, listener) {
                    listeners[eventName] = listeners[eventName] || [];
                    if (listeners[eventName].indexOf(listener) === -1) {
                        listeners[eventName].push(listener);
                    }
                },
                off: function(eventName, listener) {
                    if (Array.isArray(listeners[eventName])) {
                        var listenerIndex = listeners[eventName].indexOf(listener);
                        if (listenerIndex !== -1) {
                            listeners[eventName].splice(listenerIndex, 1);
                        }
                    }
                },
                dispatch: function({ action, args }) {
                    containerService.dispatch({ action, args });
                },
                getState: function() {
                    return containerService.get_state();
                }
            });
        });
}
