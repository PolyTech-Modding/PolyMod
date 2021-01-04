const verificationProperties = {
    Core: {
        badge: '<span class="badge bg-primary">Core Framework</span>',
        download_colour: "btn-primary",
    },
    Manual: {
        badge: '<span class="badge bg-success">Manually Verified</span>',
        download_colour: "btn-success",
    },
    Auto: {
        badge: '<span class="badge bg-warning text-dark">Auto Verified</span>',
        download_colour: "btn-warning",
        alert: `
        <div class="alert alert-warning" role="alert" id="alert_auto_verified">
            <h4 class="alert-heading mb-0 text-center">Mod <u>not</u> Verified!</h4>
            <p class="my-2">This mod has not been manually verified as safe. Download at your own risk!</p>
            <hr class="m-1">
            <p class="m-0 small"><em>You can find more information about mod verification <a href="#">here</a>.</em></p>
        </div>
        `,
    },
    Unsafe: {
        badge: '<span class="badge bg-danger">Unsafe</span>',
        download_colour: "btn-danger",
        alert: `
        <div class="alert alert-danger" role="alert" id="alert_unsafe">
            <h4 class="alert-heading mb-0 text-center">Mod <u>failed</u> verification</h4>
            <p class="my-2">This mod has failed manual verification. Download at your own risk!</p>
            <hr class="m-1">
            <p class="m-0 small"><em>You can find more information about mod verification <a href="#">here</a>.</em></p>
        </div>`,
    },
    DEFAULT: {
        badge: '<span class="badge bg-dark">Unverified</span>',
        download_colour: "btn-danger",
        alert: `
        <div class="alert alert-danger" role="alert" id="alert_unsafe">
            <h4 class="alert-heading mb-0 text-center">Mod <u>not</u> verified!</h4>
            <p class="my-2">This mod has not been manually verified yet. Download at your own risk!</p>
            <hr class="m-1">
            <p class="m-0 small"><em>You can find more information about mod verification <a href="#">here</a>.</em></p>
        </div>`,
    },
    fromVerificationLevel(verificationLevel) {
        if (this.hasOwnProperty(verificationLevel)) {
            return this[verificationLevel];
        } else {
            return this.DEFAULT;
        }
    },
};

function timeSince(timeStamp) {
    var now = new Date(),
        secondsPast = (now.getTime() - timeStamp) / 1000;
    if (secondsPast < 60) {
        return parseInt(secondsPast) + "s";
    }
    if (secondsPast < 3600) {
        return parseInt(secondsPast / 60) + "m";
    }
    if (secondsPast <= 86400) {
        return parseInt(secondsPast / 3600) + "h";
    }
    if (secondsPast > 86400) {
        day = timeStamp.getDate();
        month = timeStamp
            .toDateString()
            .match(/ [a-zA-Z]*/)[0]
            .replace(" ", "");
        year = timeStamp.getFullYear() == now.getFullYear() ? "" : " " + timeStamp.getFullYear();
        return day + " " + month + year;
    }
}

// Use a good sanitizer pls
var safetext = function (text) {
    var table = {
        "<": "lt",
        ">": "gt",
        '"': "quot",
        "'": "apos",
        "&": "amp",
        "\r": "#10",
        "\n": "#13",
    };

    return text.toString().replace(/[<>"'\r\n&]/g, function (chr) {
        return "&" + table[chr] + ";";
    });
};
const Roles = {
    Roles    : "00000000",
    OWNER    : "00000001",
    ADMIN    : "00000010",
    MOD      : "00000100",
    VERIFYER : "00001000",
    MAPPER   : "00010000",
    BOT      : "00100000",
    hasRole(role){
        if (!Roles.hasOwnProperty(role)) return false
        for (i = 0; i < 8; i++){
            if (this[role][i] == 1 && this.Roles[i] == 1) return true
        }
        return false
    }
}
var data
function setNavbarButtons(){
    fetch( "/public_api/me")
    	.then(function (response) {
    		if (response.status !== 200){
    			fetch( "/oauth2_url")
    				.then(function (response) {
    					response.json().then(function (json_data) {
    						document.getElementById("login_button").setAttribute("onclick", `document.location.href = '${json_data.url}'`);
    					})
    				}
    			)
    			return
    		}
        
    		response.json().then(function (json_data) {
                console.log(json_data)
                data = json_data
                Roles.Roles = json_data.roles.toString(2).padStart(8, 0)
    			document.getElementById("login_button").setAttribute("hidden", true)
    			document.getElementById("logout_button").removeAttribute("hidden")
                document.getElementById("user_button").removeAttribute("hidden")
                
                mod_options = document.getElementById("mod_options")
                if (Roles.hasRole("VERIFYER") && mod_options != null){
                    mod_options.removeAttribute("hidden")
                }
    		})
    	}
    )
}
setNavbarButtons()
