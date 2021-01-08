// Example starter JavaScript for disabling form submissions if there are invalid fields
(function () {
    'use strict'
  
    // Fetch all the forms we want to apply custom Bootstrap validation styles to
    var forms = $('.needs-validation')
  
    // Loop over them and prevent submission
    Array.prototype.slice.call(forms)
          .forEach(function (form) {
            form.addEventListener('submit', function (event) {
                if (!form.checkValidity()) {
                    event.preventDefault()
                    event.stopPropagation()
                }
  
                form.classList.add('was-validated')
        }, false)
    })
})()


// for some reason the form triese to submit??? maybe somehting to do with bootstrap
// fixed by preventing submit because it should never be required	
let form = document.getElementById("uploadForm")
form.addEventListener('submit', function (event) {
    event.preventDefault()
    event.stopPropagation()
})

function uploadMod(){
    if (!form.checkValidity()) return
    console.log("Uploading Mod...")
    let f = new FormData()
    
    let json_data = {}	
    
    let item
    for (var i = 0; i < form.elements.length; i++){
        item = form.elements[i]
        if (item.type == "file"){
            if (item.files.length > 0){
                f.set(item.name, item.files[0], item.files[0].name)
            }
        }
        else {
            if (item.value){
                json_data[item.name] = item.value
            }
        }
    }

    json_data.keywords = $("#keywords")[0].value.split(",")
    json_data.authors = $("#authors")[0].value.split(",")

    console.log(json_data)
    //console.log(JSON.stringify(json_data))
    let data_file = new File([JSON.stringify(json_data)], "data.json")
    f.set("data.json", data_file, data_file.name)
    //console.log(f)

    fetch("/api/upload",
        {
            method: "POST",
            mode: "same-origin",
            credentials: "include",
            headers: {
                "Authorization": data.token
            },
            body: f
            
        }
    )
    .then(
        function (response){
            console.log(response.status)
            console.log(response)
            response.text().then(function (text) {
                console.log(text)
                alerts = document.getElementById("alerts")
                alerts.innerHTML = ""
                if (response.status === 400){
                    alerts.innerHTML += `<div id="alert_verification_error" class="alert alert-danger alert-dismissible fade show" role="alert">
                        Bad Request. ${text}
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
                        Uploaded Mod Successfully!
                        <button type="button" class="btn-close" data-bs-dismiss="alert" aria-label="Close"></button>
                    </div>`
                }
                document.body.scrollTop = 0;
                document.documentElement.scrollTop = 0
            });
        }
    )
}