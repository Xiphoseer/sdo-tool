import init, { Handle } from './pkg/sdo_web.js';

async function run() {
    await init();

    const outputEl = document.getElementById("output");
    const paginationEl = document.getElementById("pagination");
    const progressEl = document.getElementById("progress");
    const inputField = document.getElementById("upload");

    // Buttons
    const addToCollectionBtn = document.getElementById('add-to-collection');
    const exportToPdfBtn = document.getElementById('export-to-pdf');

    const h = new Handle(outputEl, inputField);
    await h.init();

    /*async function uploadFile(file) {
    const buf = await file.arrayBuffer();
    const arr = new Uint8Array(buf);
    await h.stage(file.name, arr);
    }*/

    /*
    const dirHandle = await navigator.storage.getDirectory();
    console.log(dirHandle);
    for await (const [key, value] of dirHandle.entries()) {
    console.log({ key, value });
    }*/

    /*
    async function uploadFiles(field) {
    h.reset();
    for (const file of field.files) {
        await uploadFile(file);
        console.log(`Completed '${file.name}'`);

        // const handle = await dirHandle.getFileHandle(file.name, { create: true });
        // console.log("creating file");
        // const newFile = await handle.createWritable();
        // console.log("writing file");
        // await newFile.write(file);
    }
    }
    */


    inputField.addEventListener('change', (event) => {
        window.location.hash = '';
        h.on_change();
    });

    async function addToCollection() {
        await h.addToCollection();
    }

    async function exportToPdf() {
        return await h.exportToPdf();
    }

    addToCollectionBtn.addEventListener('click', addToCollection);
    exportToPdfBtn.addEventListener('click', (_event) => exportToPdf().then(pdf => {
        const url = URL.createObjectURL(pdf);
        window.open(url);
        //console.log(pdf);
        
    }).catch(console.error));

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
        const bar = progressEl.children[0];
        bar.style.width = `${percent}%`;
    }

    async function renderOne(index) {
        const blob = await h.render(index);
        setProgress((index + 1) / pageCount * 100);
        if (blob) {
            pages.push(blob);
            if (index == 0) {
            append(blob, index);
            }
            console.log("Finished page", index);
        } else {
            pages.push(undefined);
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

    async function selectPage(index) {
        const blob = pages[index];
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
        //console.log(event);
        event.preventDefault();
        selectPage(--currentPage);
    }

    function onNext(event) {
        //console.log(event);
        event.preventDefault();
        selectPage(++currentPage);
    }

    function setupPagination() {
        clearPagination();

        // Previous
        const prev = document.createElement("li");
        prev.classList.add("page-item");
        const prevLink = document.createElement("a");
        prevLink.text = "Previous";
        prevLink.href = "#";
        prevLink.classList.add("page-link");
        prevLink.addEventListener('click', onPrev);
        prev.appendChild(prevLink);
        paginationEl.appendChild(prev);

        // Next
        const next = document.createElement("li");
        next.classList.add("page-item");
        const nextLink = document.createElement("a");
        nextLink.text = "Next";
        nextLink.href = "#";
        nextLink.classList.add("page-link");
        nextLink.addEventListener('click', onNext);
        next.appendChild(nextLink);
        paginationEl.appendChild(next);
    }

    async function handle(hash) {
        clearPagination();
        await h.open(hash);
        if (h.hasActive()) {
            setupPagination();
            render();
        }
    }

    async function onHashChange(hash) {
        handle(hash)
    }
    // Init
    handle(decodeURIComponent(window.location.hash));

    addEventListener('hashchange', (event) => {
        onHashChange(decodeURIComponent(new URL(event.newURL).hash))
    })
}

run();