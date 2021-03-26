document.getElementById("username").innerText = data.discord.username
document.getElementById("profile_image").src = `https://cdn.discordapp.com/avatars/${data.user_id_string}/${data.discord.avatar}.png?size=256`

let mods_select = document.getElementById("mods_select")
let teams_select = document.getElementById("teams_select")

let mod_count = document.getElementById("mod_count")
let team_count = document.getElementById("team_count")

let mods_display = document.getElementById("mods_display")
let teams_display = document.getElementById("teams_display")
let create_team_button = document.getElementById("create_team_button")


let create_team_alerts = document.getElementById("create_team_alerts")
let join_team_alerts = document.getElementById("join_team_alerts")

function addModCard(mod) {
    //console.log(mod)
    //console.log(result.authors)
    //if (result.authors == undefined) result.authors = []
    //if (result.categories == undefined) result.categories = []
    /*
    <div class="card">
            <div class="card-body">
                <div id="content" class="card-text p-0" role="button">
                    <h5>API Switcher</h5>
                    <p class="h6 mb-1">Authors: nitsuga5124#2207</p>
                    <p class="mb-1">A mod to use servers other than the official</p>
                    <span class="badge bg-primary">Core Framework</span>
                    <span class="badge bg-secondary">Category: Utilities</span>
                    <p class="text-muted mb-0">400 downloads  -  2 days ago</p>
                </div>
            </div>
            */
    card = document.createElement("div");
    card.setAttribute("role", "button");
    card.setAttribute("onclick", `document.location.href = '/mod?name=${mod.name}&version==${mod.version}&verification=${mod.verification}'`);
    card.classList.add("card", "mx-2");

    body = document.createElement("div");
    body.classList.add("card-body");
    card.appendChild(body);

    content = document.createElement("div");
    content.classList.add("card-text", "p-0");
    body.appendChild(content);

    title = document.createElement("h5");
    title.innerHTML = safetext(mod["name"]);
    //authors = document.createElement("p");
    //authors.innerHTML = "Authors: " + safetext(result.authors.join(", "));
    //authors.classList.add("h6", "mb-1");

    description = document.createElement("p");
    description.innerHTML = safetext(mod.description);
    description.classList.add("mb-1");

    labels = document.createElement("div");
    labels.innerHTML = verificationProperties.fromVerificationLevel(mod["verification"]).badge;

    //result.categories.forEach((category) => {
    //  span = document.createElement("span");
    //  span.classList.add("badge", "bg-secondary", "m-1");
    //  span.innerHTML = category;
    //  labels.innerHTML += "";
    //  labels.appendChild(span);
    //});
    downloads_and_time = document.createElement("p");
    downloads_and_time.classList.add("text-muted", "mb-0");
    var last_updated = new Date(mod.uploaded);
    downloads_and_time.innerHTML = safetext(mod.downloads + " downloads - last updated " + timeSince(last_updated));
    downloads_and_time.innerHTML += "<span style='float: right;'>Version " + safetext(mod.version) + "</span>"

    content.appendChild(title);
    //content.appendChild(authors);
    content.appendChild(description);
    content.appendChild(labels);
    content.appendChild(downloads_and_time);

    mods_display.appendChild(card);
}
function addTeamCard(team){
    let card = document.createElement("div");
    //card.setAttribute("role", "button");
    card.classList.add("card", "mx-2");

    let body = document.createElement("div");
    body.classList.add("card-body");
    card.appendChild(body);

    let content = document.createElement("div");
    content.classList.add("card-text", "p-0");
    body.appendChild(content);

    let title = document.createElement("h5");
    title.innerHTML = safetext(team.name);

    let details = document.createElement("p")
    details.innerHTML = (team.roles & TeamRoles.OWNER ? "Owner" : (team.roles & TeamRoles.ADMIN ? "Admin" : (team.roles & TeamRoles.MOD ? "Mod" : "No permissions")))

    let inviteButton = document.createElement("button")
    inviteButton.innerText = "Invite"
    inviteButton.onclick = e => {
        getInvite(team)
    }
    inviteButton.classList.add("btn", "btn-light")
    inviteButton.setAttribute("data-bs-toggle", "modal")
    inviteButton.setAttribute("data-bs-target", "#team_invite_modal")


    //authors = document.createElement("p");
    //authors.innerHTML = "Authors: " + safetext(result.authors.join(", "));
    //authors.classList.add("h6", "mb-1");

    //result.categories.forEach((category) => {
    //  span = document.createElement("span");
    //  span.classList.add("badge", "bg-secondary", "m-1");
    //  span.innerHTML = category;
    //  labels.innerHTML += "";
    //  labels.appendChild(span);
    //});
    
    content.appendChild(title);
    content.appendChild(details)
    content.appendChild(inviteButton)
    teams_display.appendChild(card);
}

function createTeam(){
    let form = document.getElementById("create_team_form")
    if (!form.reportValidity()) return
    let f = new FormData()
    f.set("name", document.getElementById("team_name").value)
    let params = new URLSearchParams(f).toString()
    fetch("./public_api/teams/create", 
        {
            method: "post",
            headers: {
                "Content-Type":"application/x-www-form-urlencoded",
                "accept-charset":"utf-8"
            },
            body: params
        }
    ).then(response => {
        if (response.status === 200){
            response.text().then(text => {
                create_team_alerts.innerHTML += `<div class="alert alert-success alert-dismissible fade show" role="alert">
                ${text}
                <button type="button" class="btn-close" data-bs-dismiss="alert" aria-label="Close"></button>
                </div>`
            })
            initialize(true)
        }
        if (response.status === 400){
            response.text().then(text => {
                create_team_alerts.innerHTML += `<div class="alert alert-danger alert-dismissible fade show" role="alert">
                ${text}
                <button type="button" class="btn-close" data-bs-dismiss="alert" aria-label="Close"></button>
                </div>`
            })
        }
    })
}

function getInvite(team){
    let team_invite = document.getElementById("team_invite")
    team_invite.innerText = ""
    team_invite.href = ""
    let f = new FormData()
    f.set("team_id", team.id)
    let params = new URLSearchParams(f).toString()
    fetch("./public_api/teams/invite?" + params, 
        {
            method: "get",
            headers: {
                "accept-charset":"utf-8"
            },
        }
    ).then(response => {
        if (response.status === 200){
            response.text().then(text => {
                team_invite.innerText = text
                team_invite.href = text
            })
        }
        if (response.status === 400){
            response.text().then(text => {
                
            })
        }
    }).catch(error => {
        team_invite.innerText = error
        team_invite.href = ""
    })
}



function refreshMods(){
    mods_display.innerHTML = ""
    data.mods.forEach(mod => {
        addModCard(mod)
    })
    mod_count.innerText = data.mods.length
    //if (data.mods.length == 0){
    //    mods_select.classList.add("disabled")
    //}
}
refreshMods()



function refreshTeams(){
    teams_display.innerHTML = ""
    data.teams.forEach(team => {
        addTeamCard(team)
    })
    team_count.innerText = data.teams.length
    //if (data.teams.length == 0){
    //    teams_select.classList.add("disabled")
    //}
}
refreshTeams()

function reloadMe(){
    initialize(true)
    refreshMods()
    refreshTeams()
}

mods_select.onclick = e => {
    teams_display.hidden = true
    mods_display.hidden = false
    teams_select.classList.remove("active")
    mods_select.classList.add("active")
    create_team_button.hidden = true
}
teams_select.onclick = e => {
    mods_display.hidden = true
    teams_display.hidden = false
    mods_select.classList.remove("active")
    teams_select.classList.add("active")
    create_team_button.hidden = false
}