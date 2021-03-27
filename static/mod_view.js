// Mod view script to fill in information like warnings etc.
//console.log(document.getElementById("data_").innerText)
//categories, verification, files, release_date
verif_detail = document.getElementById("verification_level")
verification_level = verif_detail.innerText
files = document.getElementById("files").innerHTML.split("<br>")
uploaded_string = document.getElementById("uploaded").innerText

// set readme
let readme_filename = document.getElementById("readme_filename").innerText
if (readme_filename){
    let contents = document.getElementById("content")
    let readme = contents.innerHTML
    if (readme_filename.endsWith(".md")){
        readme = marked(readme)
    }
    else if (readme_filename.endsWith(".creole")){
        var creole = new creole()
        var div = document.createElement('div');
        creole.parse(div, readme);
        readme = div.innerHTML
    }
    else if (readme_filename.endsWith(".org") || readme_filename.endsWith(".org-mode")){
        var parser = new Org.Parser();
        var orgDocument = parser.parse(readme);
        var orgHTMLDocument = orgDocument.convert(Org.ConverterHTML, {});
        readme = orgHTMLDocument.toString()
    }
    else { // default to markdown
        readme = marked(readme)
    }
    contents.innerHTML = DOMPurify.sanitize( readme );
}


level = verificationProperties.fromVerificationLevel(verification_level);

verif_detail.innerHTML = level.badge
document.getElementById("download_button").classList.add(level.download_colour);

document.getElementById("download_button").setAttribute("onclick", `document.location.href = '${files[0]}'`);

var uploaded = new Date(uploaded_string);
document.getElementById("release_date").innerHTML = uploaded.toLocaleDateString()


if (level.alert) {
    document.getElementById("verification_warnings").innerHTML = level.alert;
}

function formUpdate(){
    is_good = document.getElementsByName("is_good")[0]
    reason = document.getElementById("reason")
    if (is_good.checked){
        if (reason.hasAttribute("required")) reason.removeAttribute("required")
        return
    }
    reason.setAttribute("required", true)
}

function submitVerification(){
    form = document.getElementById("verification_form")
    is_good = document.getElementsByName("is_good")[0]
    if (!form.reportValidity()) return

    let f = new FormData(form)
    
    let checksum = files[0].split("/")
    checksum = checksum[checksum.length-1]
    f.set("is_good", is_good.checked)
    f.set("checksum", checksum)
    let params = new URLSearchParams(f).toString()
    
    fetch("./api/verify?" + params, 
        {
            mode: "same-origin",
            credentials: "include",
            headers: {
                "Authorization": getToken()
            }
        }
    ).then(
        function (response){
            //console.log(response.status)
            //console.log(response)
            alerts = document.getElementById("alerts")
            alerts.innerHTML = ""
            if (response.status === 400){
                alerts.innerHTML += `<div id="alert_verification_error" class="alert alert-danger alert-dismissible fade show" role="alert">
                    Bad Request. You may have already verified this mod or the mod might already be verified.
                    <button type="button" class="btn-close" data-bs-dismiss="alert" aria-label="Close"></button>
                </div>`
            }
            if (response.status === 401){
                alerts.innerHTML += `<div id="alert_verification_error" class="alert alert-danger alert-dismissible fade show" role="alert">
                    Insufficient Permissions.
                    <button type="button" class="btn-close" data-bs-dismiss="alert" aria-label="Close"></button>
                </div>`
            }
            if (response.status === 200){
                alerts.innerHTML += `<div id="alert_verification_success" class="alert alert-success alert-dismissible fade show" role="alert">
                    Sent Verification! You can now close this window.
                    <button type="button" class="btn-close" data-bs-dismiss="alert" aria-label="Close"></button>
                </div>`
            }
        }
    )

}

function configureTransferWindow(){
    let transfer_target = document.getElementById("transfer_target")
    while (transfer_target.options.length > 0){
        transfer_target.remove(0)
    }
    data.teams.forEach(team => {
        let option = document.createElement("option")
        option.text = team.name
        option.value = team.id
        transfer_target.add(option)
    })

}

function transferMod(){
    let transfer_target = document.getElementById("transfer_target")
    let alerts = document.getElementById("transfer_alerts")
    let f = new FormData()
    f.set("team_id", transfer_target.selectedOptions[0].value)
    f.set("mod", document.getElementById("mod_name").textContent)
    let params = new URLSearchParams(f).toString()
    fetch("./public_api/teams/transfer_mod?" + params, 
        {
            method: "get",
            headers: {
                "accept-charset":"utf-8"
            },
        }
    ).then(response => {
        if (response.status === 200){
            response.text().then(text => {
                alerts.innerHTML += `<div class="alert alert-success alert-dismissible fade show" role="alert">
                ${text}
                <button type="button" class="btn-close" data-bs-dismiss="alert" aria-label="Close"></button>
                </div>`
            })
        }
        if (response.status === 400 || response.status === 401 || response.status === 403){
            response.text().then(text => {
                alerts.innerHTML += `<div class="alert alert-danger alert-dismissible fade show" role="alert">
                ${text}
                <button type="button" class="btn-close" data-bs-dismiss="alert" aria-label="Close"></button>
                </div>`
            })
        }
    })
}

function yankMod(){
    let checksum = document.getElementById("files").textContent.split("/")[3]
    let alerts = document.getElementById("yank_alerts")
    let f = new FormData()
    f.set("checksum", checksum)
    f.set("reason", document.getElementById("yank_reason").value)
    let params = new URLSearchParams(f).toString()
    fetch("./api/yank?" + params,
        {
            mode: "same-origin",
            credentials: "include",
            headers: {
                "Authorization": getToken()
            }
        }
    ).then(response => {
        if (response.status === 200){
            response.text().then(text => {
                alerts.innerHTML += `<div class="alert alert-success alert-dismissible fade show" role="alert">
                ${text}
                <button type="button" class="btn-close" data-bs-dismiss="alert" aria-label="Close"></button>
                </div>`
            })
        }
        if (response.status === 400 || response.status === 401 || response.status === 403){
            response.text().then(text => {
                alerts.innerHTML += `<div class="alert alert-danger alert-dismissible fade show" role="alert">
                ${text}
                <button type="button" class="btn-close" data-bs-dismiss="alert" aria-label="Close"></button>
                </div>`
            })
        }
    })
}