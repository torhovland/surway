use seed::prelude::wasm_bindgen;

#[wasm_bindgen]
extern "C" {
    pub type GeolocationCoordinates;

    #[wasm_bindgen(method, getter)]
    pub fn latitude(this: &GeolocationCoordinates) -> f64;

    #[wasm_bindgen(method, getter)]
    pub fn longitude(this: &GeolocationCoordinates) -> f64;

    #[wasm_bindgen(method, getter)]
    pub fn altitude(this: &GeolocationCoordinates) -> f64;

    #[wasm_bindgen(method, getter)]
    pub fn accuracy(this: &GeolocationCoordinates) -> f64;

    #[wasm_bindgen(method, getter, js_name = "altitudeAccuracy")]
    pub fn altitude_accuracy(this: &GeolocationCoordinates) -> f64;

    #[wasm_bindgen(method, getter)]
    pub fn heading(this: &GeolocationCoordinates) -> f64;

    #[wasm_bindgen(method, getter)]
    pub fn speed(this: &GeolocationCoordinates) -> f64;

    pub type GeolocationPosition;

    #[wasm_bindgen(method, getter)]
    pub fn coords(this: &GeolocationPosition) -> GeolocationCoordinates;
}
