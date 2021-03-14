import '../styles/index.scss';
const rust = import("../pkg/index.js");

rust.then(r => {
    const [set_latitude, set_longitude] = r.start_seed();
    set_latitude(42.0);

    r.greet('Hi from index.js');

    function success(pos) {
        const crd = pos.coords;
        console.log(crd);

        set_latitude(crd.latitude);
        set_longitude(crd.longitude);
    }

    function error(err) {
        console.error('Geolocation error(' + err.code + '): ' + err.message);
    }

    if (!navigator.geolocation) {
        console.error('Geolocation is not supported by your browser');
    } else {
        navigator.geolocation.watchPosition(success, error);
    }

}).catch(console.error);
