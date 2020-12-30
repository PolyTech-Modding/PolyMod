/*
TODO: change search results to be in main body (remove the card container)
	so that there is only 1 scroll bar
Search Results script to:
- set filters to the values searched for
- handle search submisson
- add the cards for the search results
- infinite scroll
*/

const ADD_NEW_CARD_THRESHOLD = 500;
let is_loading = false;
let is_end = false;
results = document.getElementById("search_results");
wrapper = document.getElementById("search_wrapper");
let after_checksum = null;
let per_page = 30
/*
<div class="d-flex justify-content-center">
		<div class="spinner-border text-primary" role="status">
			<span class="visually-hidden">Loading...</span>
		</div>
	</div>
*/
loading_wheel = document.createElement("div");
loading_wheel.classList.add("d-flex", "justify-content-center");
spinner = document.createElement("div");
spinner.classList.add("spinner-border", "text-primary", "m-4");
spinner.setAttribute("role", "status");
span = document.createElement("spam");
span.classList.add("visually-hidden");
span.innerText = "Loading...";
spinner.appendChild(span);
loading_wheel.appendChild(spinner);

function handleSearch(clear = true, after_checksum = null) {
  form = document.getElementById("main_form");
  search_query = document.getElementById("query").cloneNode();
  search_query.setAttribute("hidden", true);
  search_field = document.getElementsByName("search_field")[0];
  verification = document.getElementsByName("verification")[0]
  let f = new FormData(form);

  // add query to form and get the names_only and keywords_only parameters from search_field
  f.set("query", search_query.value);

  let frontend_search_string = new URLSearchParams(f);
  frontend_params = frontend_search_string.toString();

  if (search_field.value == "name") {
    f.set("names_only", true);
  }
  if (search_field.value == "keywords") {
    f.set("keywords_only", true);
  }
  if (verification.value == ""){
    f.delete("verification")
  }
  f.delete("search_field");
  f.set("per_page", per_page);

  if (after_checksum) {
    f.set("after", after_checksum);
  }
  let api_search_string = new URLSearchParams(f);

  api_params = api_search_string.toString();

  window.history.replaceState({}, "", window.location.pathname + "?" + frontend_params);

  results.appendChild(loading_wheel);
  fetch("./public_api/search?" + api_params, {mode: "no-cors"})
    .then(function (response) {
      console.log(results.status)
      if (response.status !== 200) {
        console.log("Looks like there was a problem. Status Code: " + response.status);
        return;
      }
      // Examine the text in the response
      response.json().then(function (json_data) {
        data = json_data;
        if (clear) {
          console.log("Clearing search results.");
          results.innerHTML = "";
        }
        if (data.length > 0){
          if (after_checksum == data[0].checksum){
            console.log("Removing leading duplicate entry")
            data = data.slice(1)
          }
        }
        data.forEach((result) => {
          addCard(result);
        });
        
        if (data.length == 0 || data.length < per_page-1){
          // if page isn't full, there won't be another page
          console.log("Page is emtpy or not full, end of resuls reached.")
          is_end = true
          results.innerHTML += '<div class="d-flex justify-content-center m-2 h5">You\'ve reached the end.</div>';
        }
        else {
          globalThis.after_checksum = data[data.length - 1].checksum;
        }
        is_loading = false;
      });
    })
    .catch(function (err) {
      console.log("Fetch Error :-S", err);
    })
    .then(function () {
      results.removeChild(loading_wheel);
    });
}

function optionContains(option, value) {
  for (i = 0; i < option.options.length; ++i) {
    if (option.options[i].value == value) {
      return true;
    }
  }
  return false;
}
function fillFormItemFromUrl(name, html_item, type, params){
  if (type == "input"){
    if (params.has(name)) {
      html_item.value = decodeURI(params.get(name));
    }
  }
  if (type == "option"){
    if (params.has(name)) {
      if (optionContains(html_item, params.get(name))) {
        html_item.value = params.get(name);
      }
    }
  }
  if (type == "switch"){
    if (params.has(name)) {
      html_item.setAttribute("checked", true);
    }
  }
}




function autoFillFromUrl() {
  params = new URLSearchParams(location.search);

  fillFormItemFromUrl("query", document.getElementById("query"), "input", params)
  fillFormItemFromUrl("category", document.getElementsByName("category")[0], "option", params)
  fillFormItemFromUrl("sort_by", document.getElementsByName("sort_by")[0], "option", params)
  fillFormItemFromUrl("search_field", document.getElementsByName("search_field")[0], "option", params)
  fillFormItemFromUrl("verification", document.getElementsByName("verification")[0], "option", params)
  fillFormItemFromUrl("reverse", document.getElementsByName("reverse")[0], "switch", params)
}

function addCard(result) {
  console.log(result)
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
  card.setAttribute("onclick", `document.location.href = '/mod?name=${result.name}'`);
  card.classList.add("card");

  body = document.createElement("div");
  body.classList.add("card-body");
  card.appendChild(body);

  content = document.createElement("div");
  content.classList.add("card-text", "p-0");
  body.appendChild(content);

  title = document.createElement("h5");
  title.innerHTML = safetext(result["name"]);
  //authors = document.createElement("p");
  //authors.innerHTML = "Authors: " + safetext(result.authors.join(", "));
  //authors.classList.add("h6", "mb-1");

  description = document.createElement("p");
  description.innerHTML = safetext(result.description);
  description.classList.add("mb-1");

  labels = document.createElement("div");
  labels.innerHTML = verificationProperties.fromVerificationLevel(result["verification"]).badge;

  //result.categories.forEach((category) => {
  //  span = document.createElement("span");
  //  span.classList.add("badge", "bg-secondary", "m-1");
  //  span.innerHTML = category;
  //  labels.innerHTML += "";
  //  labels.appendChild(span);
  //});
  downloads_and_time = document.createElement("p");
  downloads_and_time.classList.add("text-muted", "mb-0");
  var last_updated = new Date(result.uploaded);
  downloads_and_time.innerHTML = safetext(result.downloads + " downloads - last updated " + timeSince(last_updated));
  downloads_and_time.innerHTML += "<span style='float: right;'>Version " + safetext(result.version) + "</span>"

  content.appendChild(title);
  //content.appendChild(authors);
  content.appendChild(description);
  content.appendChild(labels);
  content.appendChild(downloads_and_time);

  results.appendChild(card);
}

autoFillFromUrl();
//createPageButtons()
handleSearch();

// descrease search result wrapper div height by 1 so you can scroll which initiates
// infinite scroll loading

// this is the scroll event handler
function scroller() {
  if (is_loading || is_end) return;
  // if near the bottom of scroll: get next results
  var body = document.body,
    html = document.documentElement;

  var documentHeight = Math.max(body.scrollHeight, body.offsetHeight, html.clientHeight, html.scrollHeight, html.offsetHeight);
  if (window.scrollY + window.innerHeight + 15 > documentHeight) {
    is_loading = true;
    console.log("Fetching more results");
    handleSearch(false, globalThis.after_checksum);
  }
}
// hook the scroll handler to scroll event
window.addEventListener("scroll", scroller);
