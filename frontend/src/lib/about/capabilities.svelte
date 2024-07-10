<section class="capabilities">
  <div class="container block-container ext-container">
    <div class="row">
      <div class="col-xs-8 col-md-4">
        <div class="well">
          <ul class="nav nav-list">
            <li class="nav-header" style="color:fuchsia;">My links</li>
            <li class="divider"></li>
            <li class="active bio-link"><a href="https://github.com/UserCommon">UserCommon</a></li>
            <li class="bio-link"><a href="https://t.me/UsCmn">@UsCmn</a></li>
            <li class="bio-link"><a href="https://vk.com/shardoffsky">@shardoffsky</a></li>
          </ul>
        </div>
      </div>
      <div class="col-xs-8 col-md-8">
        <div style="display: block !important;">
          <h1 class="lead page-header">My capabilities</h1>
          <br>
          <table class="table table-bordered table-striped">
            <thead>
              <tr style="text-transform: uppercase;">
                <th>
                  <h1>Technology</h1><br>
                </th>
                <th>
                  <h1>Level</h1><br>
                </th>
              </tr>
            </thead>
            <tbody>
              <tr>
                <td>
                  <span class="label label-important">Rust</span>
                </td>
                <td>
                  <code id="hashContainerRust"></code>
                </td>
              </tr>
              <tr>
                <td>
                  <span class="label label-success">Django</span>
                </td>
                <td>
                  <code id="hashContainerDjango"></code>
                </td>
              </tr>
              <tr>
                <td>
                  <span class="label label-success">Python</span>
                </td>
                <td>
                  <code id="hashContainerPython"></code>
                </td>
              </tr>
              <tr>
                <td>
                  <span class="label label-success">Linux</span>
                </td>
                <td>
                  <code id="hashContainerLinux"></code>
                </td>
              </tr>
              <tr>
                <td>
                  <span class="label label-success">SQL</span>
                </td>
                <td>
                  <code id="hashContainerSQL"></code>
                </td>
              </tr>
              <tr>
                <td>
                  <span class="label label-success">Git</span>
                </td>
                <td>
                  <code id="hashContainerGit"></code>
                </td>
              </tr>
              <tr>
                <td>
                  <span class="label label-success">Docker</span>
                </td>
                <td>
                  <code id="hashContainerDocker"></code>
                </td>
              </tr>
              <tr>
                <td>
                  <span class="label label-success">Svelte</span>
                </td>
                <td>
                  <code id="hashContainerSvelte"></code>
                </td>
              </tr>
            </tbody>
          </table>
          <script>
            // Function to calculate and append hashes based on screen width
            function hashResize(id, lvl) {
              const fraction = 10
              const screenWidth = window.innerWidth; // Get the viewport width
              const hashCount = Math.floor(screenWidth / 50) / fraction * lvl;
              const hashContainer = document.getElementById(id);
              hashContainer.textContent = '#'.repeat(hashCount);
            }
            function updateHashes() {
              hashResize('hashContainerRust', 7)
              hashResize('hashContainerDjango', 5)
              hashResize('hashContainerPython', 6)
              hashResize('hashContainerLinux', 6)
              hashResize('hashContainerSQL', 3)
              hashResize('hashContainerGit', 4)
              hashResize('hashContainerDocker', 3)
              hashResize('hashContainerSvelte', 3)
            }
            // Call the function initially and whenever the window is resized
            updateHashes();
            window.addEventListener('resize', updateHashes);
          </script>
        </div>
        </div>
      </div>
  </div>
</section>
