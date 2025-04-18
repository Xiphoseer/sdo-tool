import init, { Handle } from './pkg/sdo_web.js';

function onError(e) {
    console.error(e);
    alert(e instanceof Error ? `ERROR ${e.name}: ${e.message}` : e);
}

async function run() {
    await init();

    const outputEl = document.getElementById("output");
    const paginationEl = document.getElementById("pagination");
    const progressEl = document.getElementById("progress");
    const inputField = document.getElementById("upload");

    // Buttons
    const addToCollectionBtn = document.getElementById('add-to-collection');
    const exportToPdfBtn = document.getElementById('export-to-pdf');
    const formatSelector = document.getElementById('format-select');
    const toolArea = document.getElementById('tool-area');
    const uploadArea = document.getElementById('upload-area');

    const navLinks = document.querySelectorAll('.nav-item, .navbar-brand')
    const menuToggle = document.getElementById('navbarNav')
    const bsCollapse = bootstrap.Collapse.getOrCreateInstance(menuToggle, {toggle: false})
    navLinks.forEach((l) => {
        l.addEventListener('click', () => { 
            if (menuToggle.classList.contains('show')) {
                bsCollapse.toggle()
            }
        })
    })

    const h = new Handle(outputEl, inputField);
    await h.init().catch(onError);

    const STAGED_PATH = '/staged/';

    async function onInputFieldChange(_event) {
        const oldHash = window.location.hash;
        const oldPath = oldHash ? oldHash.slice(1) : '/'; // remove leading #
        const isStagedPath = oldPath === STAGED_PATH;
        console.log('input field changed', oldPath);
        if (!isStagedPath) {
            console.debug("Navigating to", STAGED_PATH);
            window.location.hash = STAGED_PATH;
        } else {
            console.debug("Triggering change");
            await h.onChange().catch(onError);
        }
    }

    inputField.addEventListener('change', onInputFieldChange);

    async function addToCollection() {
        await h.addToCollection().then((count) => {
            alert(`Added ${count} elements to collection!`)
        });
    }

    async function exportToPdf() {
        return await h.exportToPdf();
    }

    addToCollectionBtn.addEventListener('click', addToCollection);
    exportToPdfBtn.addEventListener('click', (_event) => exportToPdf().then(pdf => {
        const url = URL.createObjectURL(pdf);
        window.open(url);
    }).catch(onError));

    let pages = [];
    let pageCount = 0;

    function makePageListItem(blob, text) {
        const p = document.createElement("p");
        p.textContent = text;
        const img = document.createElement("img");
        img.src = URL.createObjectURL(blob);
        img.classList.add("container-fluid");
        const listItem = document.createElement("div");
        listItem.classList.add("list-group-item");
        listItem.appendChild(p);
        listItem.appendChild(img);
        return listItem;
    }

    function updatePageListItem(blob, text) {
        const li = outputEl.children[1];
        console.log("li", li);
        const span = li.children[0];
        span.textContent = text;
        const img = li.children[1];
        console.log("img", img);
        img.src = URL.createObjectURL(blob);
    }

    function pageIndicatorText(index) {
        return `Page ${index + 1} / ${pageCount}`;
    }

    async function append(blob, index) {
        const listItem = makePageListItem(blob, pageIndicatorText(index));
        outputEl.appendChild(listItem);
    }

    function setProgress(percent) {
        if (percent < 100) {
            progressEl.style.removeProperty('display'); // reset display: none
        } else {
            clearProgress();
        }
        const bar = progressEl.children[0];
        bar.style.width = `${percent}%`;
    }

    function clearProgress() {
        progressEl.style.display = 'none';
    }

    function clearFormatSelect() {
        formatSelector.classList.add('d-none');

        uploadArea.classList.add('col-sm-10');
        uploadArea.classList.remove('col-sm-8');

        //toolArea.classList.add('col-sm-2');
        //toolArea.classList.remove('col-sm-1');
    }

    function setupFormatSelect() {
        formatSelector.classList.remove('d-none');
        uploadArea.classList.add('col-sm-8');
        uploadArea.classList.remove('col-sm-10');

        //toolArea.classList.add('col-sm-1');
        //toolArea.classList.remove('col-sm-2');
    }

    async function renderOne(index) {
        const blob = await h.render(index);
        setProgress((index + 1) / pageCount * 100);
        if (blob) {
            pages.push(blob);
            updatePagination();
            if (index == 0) {
                append(blob, index);
            }
            console.log("Finished page", index);
        } else {
            pages.push(undefined);
            updatePagination();
            console.log("Empty page", index);
        }
        const nextIndex = index + 1;
        if (nextIndex < pageCount) {
            setTimeout(() => renderOne(nextIndex));
        }
    }

    async function render() {
        pages = [];
        pageCount = h.activePageCount();
        setProgress(0);
        await renderOne(0);
    }

    function updatePagination() {
        const prev = document.getElementById('page-prev');
        const next = document.getElementById('page-next');

        // check with the real number of rendered pages
        if (currentPage + 1 < pages.length) {
            next.classList.remove("disabled");
        } else {
            next.classList.add("disabled");
        }

        if (currentPage > 0) {
            prev.classList.remove("disabled");
        } else {
            prev.classList.add("disabled");
        }
    }

    async function selectPage(index) {
        const blob = pages[index];
        currentPage = index;
        updatePagination();
        if (blob) {
            updatePageListItem(blob, pageIndicatorText(index));
        }
    }

    let currentPage = 0;

    function clearPagination() {
        paginationEl.innerHTML = "";
        currentPage = 0;
    }

    function onPrev(event) {
        event.preventDefault();
        selectPage(--currentPage);
    }

    function onNext(event) {
        event.preventDefault();
        selectPage(++currentPage);
    }

    function setupPagination() {
        clearPagination();

        // Previous
        const prev = document.createElement("li");
        prev.id = "page-prev";
        prev.classList.add("page-item");
        const prevLink = document.createElement("a");
        prevLink.id = "prev-link";
        prevLink.text = "Previous";
        prevLink.href = "#";
        prevLink.classList.add("page-link");
        prevLink.addEventListener('click', onPrev);
        prev.appendChild(prevLink);
        paginationEl.appendChild(prev);

        // Next
        const next = document.createElement("li");
        next.id = "page-next";
        next.classList.add("page-item");
        const nextLink = document.createElement("a");
        nextLink.id = "next-link";
        nextLink.text = "Next";
        nextLink.href = "#";
        nextLink.classList.add("page-link");
        nextLink.addEventListener('click', onNext);
        next.appendChild(nextLink);
        paginationEl.appendChild(next);
    }

    async function handle(hash) {
        clearPagination();
        clearProgress();
        clearFormatSelect();
        await h.open(hash);
        if (h.hasActive()) {
            setupPagination();
            setupFormatSelect();
            render();
        }
    }

    async function onHashChange(hash) {
        await handle(hash)
    }

    addEventListener('hashchange', (event) => {
        onHashChange(decodeURIComponent(new URL(event.newURL).hash))
    })

    // Init
    await handle(decodeURIComponent(window.location.hash)).catch(onError);
}

run();