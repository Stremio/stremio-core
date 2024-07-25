function getId() {
    return Math.random().toString(32).slice(2);
}

function Bridge(scope, handler) {
    handler.addEventListener('message', async ({ data: { request } }) => {
        if (!request) return;

        const { id, path, args } = request;
        try {
            const value = path.reduce((value, prop) => value[prop], scope);
            let data;
            if (typeof value === 'function') {
                const thisArg = path.slice(0, path.length - 1).reduce((value, prop) => value[prop], scope);
                data = await value.apply(thisArg, args);
            } else {
                data = await value;
            }

            handler.postMessage({ response: { id, result: { data } } });
        } catch (error) {
            handler.postMessage({ response: { id, result: { error } } });
        }
    });

    this.call = async (path, args) => {
        const id = getId();
        return new Promise((resolve, reject) => {
            const onMessage = ({ data: { response } }) => {
                if (!response || response.id !== id) return;

                handler.removeEventListener('message', onMessage);
                if ('error' in response.result) {
                    reject(response.result.error);
                } else {
                    resolve(response.result.data);
                }
            };
            handler.addEventListener('message', onMessage);
            handler.postMessage({ request: { id, path, args } });
        });
    };
}

module.exports = Bridge;
