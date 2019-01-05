enum Action { Init }

enum Loadable<T> {
    NotLoaded,
    Loading,
    Ready(T)
}
