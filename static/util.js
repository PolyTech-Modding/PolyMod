let $ = (s) => {return document.querySelectorAll(s)}
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
    Yanked: {
        badge: '<span class="badge bg-dark">Yanked</span>',
        download_colour: "btn-danger",
        alert: `
        <div class="alert alert-danger" role="alert" id="alert_unsafe">
            <h4 class="alert-heading mb-0 text-center">Mod <u>failed</u> verification</h4>
            <p class="my-2">This mod has been yanked by the owner. A download is still available but not recommended!</p>
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
const TeamRoles = {
    OWNER    : 0b00000001,
    ADMIN    : 0b00000010,
    MOD      : 0b00000100,
}

var data, user_data, oauth2_url, logged_in


function tryGetToken(){
    if ((!localStorage.token || localStorage.token == "null") && localStorage.logged_in == "true"){
        fetch("/token")
            .then(function (response) {
                if (response.status !== 200){
                    //console.log(response.status)
                    //localStorage.logged_in = false
                    return
                }
            
                response.text().then(function (text) {
                    //console.log(json_data)
                    localStorage.token = text
                })
            }
        )
    }
}


function initialize(forceReload=false){
    //localStorage.me
    //localStorage.oauth2_url
    //localStorage.logged_in
    //localStorage.as_team
    //localStorage.token
    //localStorage.team_id
    //localStorage.team_token
    if (localStorage.as_team == undefined){
        localStorage.as_team = false
    }
    
    if (localStorage.me && !forceReload){ // if user_data is cached
        setup(JSON.parse(localStorage.me))
    }
    else { // if not cached, fetch user data
        fetch("/public_api/me")
            .then(function (response) {
                if (response.status !== 200){
                    console.log(response.status)
                    localStorage.logged_in = false
                    return
                }
            
                response.json().then(function (json_data) {
                    //console.log(json_data)
                    localStorage.logged_in = true
                    localStorage.me = JSON.stringify(json_data)
                    tryGetToken()
                    setup(json_data)
                })
            }
        )
    }
    tryGetToken()
    
    if (!localStorage.logged_in || localStorage.logged_in == "false"){
        localStorage.token = "null"
        if (!localStorage.oauth2_url){
            fetch("/oauth2_url")
                .then(function (response) {
                    response.json().then(function (res) {
                        console.log(res)
                        localStorage.oauth2_url = res.url
                        document.getElementById("login_button").setAttribute("onclick", `document.location.href = '${localStorage.oauth2_url}'`);
                    })
                }
            )
        }
        else {
            document.getElementById("login_button").setAttribute("onclick", `document.location.href = '${localStorage.oauth2_url}'`);
        }
    }
}

function setup(json_data){
    data = json_data
    Roles.Roles = json_data.roles.toString(2).padStart(8, 0)
    document.getElementById("login_button").setAttribute("hidden", true)
    document.getElementById("logout_button").removeAttribute("hidden")
    document.getElementById("upload_mods_nav_item").removeAttribute("hidden")
    document.getElementById("logout_button").setAttribute("onclick", `localStorage.clear(); window.location.href = '/logout'`)
    document.getElementById("user_button").removeAttribute("hidden")
    document.getElementById("user_button").innerHTML = `
        <span class="">
            <img class="me-2" src="https://cdn.discordapp.com/avatars/${json_data.user_id_string}/${json_data.discord.avatar}.png?size=256" style='width: 32px; height: 32px;'>
            ${json_data.discord.username}
        </span>`
    
    mod_options = document.getElementById("mod_options")
    if (Roles.hasRole("VERIFYER") && mod_options != null){
        mod_options.removeAttribute("hidden")
    }
    let authors = document.getElementById("authors")
    if (authors) {
        authors.value = json_data.discord.username
        $('input[type="tags"]').forEach(tagsInput)
    }
    let team_switcher = document.getElementById("team_switcher")
    //console.log(team_switcher)
    while (team_switcher.options.length > 1){
        team_switcher.remove(1)
    }
    data.teams.forEach(team => {
        let option = document.createElement("option")
        option.text = team.name
        option.value = team.id
        team_switcher.add(option)
    })

    if (localStorage.as_team == "true"){
        for (var i = 0; i < team_switcher.options.length; i++){
            if (team_switcher.options[i].value == localStorage.team_id){
                team_switcher.selectedIndex = i
            }
        }
        if (!localStorage.team_token || localStorage.team_token == "null"){
            teamSwitch()
        }
    }
}

function getToken(){
    return localStorage.as_team == "true" ? localStorage.team_token : localStorage.token
}

function teamSwitch(){
    let team_switcher = document.getElementById("team_switcher")
    let team_id = team_switcher.selectedOptions[0].value
    let team_name = team_switcher.selectedOptions[0].text
    if (team_id == "none"){
        localStorage.as_team = false
        localStorage.team_token = ""
        localStorage.team_id = ""
    }
    else {
        console.log(`Switching to team ${team_name}`)
        let f = new FormData()
        f.set("id", team_id)
        let params = new URLSearchParams(f).toString()
        fetch("./public_api/teams/token?" + params, 
            {
                method: "get",
                headers: {
                    "accept-charset":"utf-8"
                },
            }
        ).then(response => {
            if (response.status === 200){
                response.text().then(text => {
                    localStorage.as_team = true
                    localStorage.team_id = team_id
                    localStorage.team_token = text
                    //location.reload()
                })
            }
        })
    }
}

initialize()
