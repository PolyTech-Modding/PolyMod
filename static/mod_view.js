// Mod view script to fill in information like warnings etc.
//console.log(document.getElementById("data_").innerText)
//categories, verification, files, release_date
verif_detail = document.getElementById("verification_level")
verification_level = verif_detail.innerText
files = document.getElementById("files").innerHTML.split("<br>")
uploaded_string = document.getElementById("uploaded").innerText
// set readme
document.getElementById("content").innerHTML = marked(document.getElementById("content").innerHTML);

level = verificationProperties.fromVerificationLevel(verification_level);

verif_detail.innerHTML = level.badge
document.getElementById("download_button").classList.add(level.download_colour);

document.getElementById("download_button").setAttribute("onclick", `document.location.href = '${files[0]}'`);

var uploaded = new Date(uploaded_string);
document.getElementById("release_date").innerHTML = uploaded.toLocaleDateString()


if (level.alert) {
  document.getElementById("verification_warnings").innerHTML = level.alert;
}
