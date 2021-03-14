import { greet } from '/pkg';

export function name() {
    return 'Rust';
};

export class MyClass {
    constructor() {
        this._number = 42;
    }

    get number() {
        return this._number;
    }

    set number(n) {
        return this._number = n;
    }

    render() {
        return `My number is: ${this.number}`;
    }
};

function sleep(ms) {
    var unixtime_ms = new Date().getTime();
    while (new Date().getTime() < unixtime_ms + ms) { }
}

export class GeoLocator {
    get latitude() {
        return this._latitude;
    }

    get longitude() {
        return this._longitude;
    }

    locate() {
        console.log('Hello from JS locate!');

        greet('Tor');

        var success = (function (position) {
            this._latitude = position.coords.latitude;
            this._longitude = position.coords.longitude;
            console.log(`Latitude: ${this._latitude} °, Longitude: ${this._longitude} °`);
        }).bind(this);

        function error() {
            console.log('Unable to retrieve your location');
        }

        if (!navigator.geolocation) {
            console.log('Geolocation is not supported by your browser');
        } else {
            console.log('Locating…');
            navigator.geolocation.getCurrentPosition(success, error);
        }

        return 'foo';
    }
};
