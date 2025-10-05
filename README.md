
<!-- Improved compatibility of back to top link: See: https://github.com/othneildrew/Best-README-Template/pull/73 -->
<a id="readme-top"></a>
<!--
*** Thanks for checking out the Best-README-Template. If you have a suggestion
*** that would make this better, please fork the repo and create a pull request
*** or simply open an issue with the tag "enhancement".
*** Don't forget to give the project a star!
*** Thanks again! Now go create something AMAZING! :D
-->



<!-- PROJECT SHIELDS -->
<!--
*** I'm using markdown "reference style" links for readability.
*** Reference links are enclosed in brackets [ ] instead of parentheses ( ).
*** See the bottom of this document for the declaration of the reference variables
*** for contributors-url, forks-url, etc. This is an optional, concise syntax you may use.
*** https://www.markdownguide.org/basic-syntax/#reference-style-links
-->
[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]
[![project_license][license-shield]][license-url]
[![LinkedIn][linkedin-shield]][linkedin-url]



<!-- PROJECT LOGO -->
<br />
<div align="center">
  <a href="https://github.com/suri-codes/TARS">
    <!-- TODO: later -->
    <!-- <img src="images/logo.png" alt="Logo" width="80" height="80"> -->
  </a>

<h3 align="center">TARS</h3>
An opinionated take on task management.

  <!-- <p align="center"> -->
    <!-- <br /> -->
    <!-- <a href="https://github.com/suri-codes/TARS"><strong>Explore the docs »</strong></a> -->
    <!-- <br /> -->
    <!-- <br /> -->
    <!-- <a href="https://github.com/suri-codes/TARS">View Demo</a> -->
    <!-- &middot; -->
    <!-- <a href="https://github.com/suri-codes/TARS/issues/new?labels=bug&template=bug-report---.md">Report Bug</a> -->
    <!-- &middot; -->
    <!-- <a href="https://github.com/suri-codes/TARS/issues/new?labels=enhancement&template=feature-request---.md">Request Feature</a> -->
  <!-- </p> -->
</div>

<!-- ABOUT THE PROJECT -->
## About The Project

[![TARS Tui][product-screenshot]](https://suri.codes/tars/full.png)

I've been thinking about writing highly personal solutions that solve my problems
precisely, rather than being satisfied with off-the-shelf tools.  I'm the kind of
person who works on 5 different things at the same time, and keeping track of what
I'm trying to do with each project while also balancing school assignments is a
chore. So, I decided to make some software that would help me keep track of
short-term and long-term goals while also being quick and easy to use!

<!-- GETTING STARTED -->
## Getting Started

### Prerequisites
This project only requires Rust, preferably the latest release.

### Installation

I plan to distribute prebuilt binaries in the future, but for now, you can do the following to build and install locally.

1. Clone the repo

   ```sh
   git clone https://github.com/suri-codes/TARS.git
   ```
2. Run the Daemon
   ```sh
   cargo run --bin tars-daemon
   ```
  If you want to use TARS long-term, I recommend building the binary and adding this to your startup script. The TARS-TUI will not start without the daemon running. 
3. Run the TUI
   ```sh
  cargo run --bin tars-tui
   ```
  You can also alias this in your shell as `tars`.

<!-- USAGE EXAMPLES -->
## Usage

### The Explorer
  <img src="https://suri.codes/tars/explorer.png" alt="Image of the TARS Explorer" />

  The explorer is the main component of TARS and shows you all the tasks and
  groups you've created, organized into a tree structure.  Interacting with this
  component is the standard way of creating new tasks and groups. If it wasn't clear
  already, solid colored rows are groups, while rows with colored text are
  tasks.

  <img src="https://suri.codes/tars/explorer_scoped.png" alt="Image of the TARS Todo-List inside a scope" />
  The Explorer is responsible for setting the scope that the TodoList sources
  to sort active tasks by priority. You can step into a group to set it as the
  scope, resulting in the explorer only displaying tasks from that group.  You
  can use the breadcrumbs at the bottom to see how deep inside the TARS tree
  you've traversed. Of course, you can step out of a scope one by one till
  you're back at home.


### The TodoList

  <img src="https://suri.codes/tars/todolist.png" alt="Image of the TARS Todo-List" />
  The TodoList is how you can check the priority order (descending) of
  the tasks in the selected scope. It uses a custom priority calculation
  algorithm that takes into account the priority of all ancestor groups of a
  task, its own priority, and how close the due date is.  After calculating this
  for all tasks in the current scope, it orders them from most to least
  important.

  
### The Inspector  
  <img src="https://suri.codes/tars/inspector.png" alt="Image of the TARS Inspector" />

  The Inspector is the interface to 

  - Change task/group name
  - Change task/group priority
  - Change task description
  - Change task completion status
  - Change task due date
  - Change group color

  The inspector's displayed item changes based on the focus of the last selected item in either the TodoList or Explorer. 

  <img src="https://suri.codes/tars/description.png" alt="Image of the TARS Description Edit Split View" />

  Another cool feature is editing the description! If you're using a supported system editor / terminal multiplexer combo (currently only helix + zellij),
  you can edit your text with terminal rendered markdown on the side (thanks to [glow](https://github.com/charmbracelet/glow))! 

### Keybindings

You can specify your own keybinds in the config file at `~/.config/tars/config.toml`. 

There isn't any documentation on keybindings right now, but take a look at the [default bindings](https://github.com/suri-codes/TARS/blob/readme/tars-tui/.config/config.toml).


<!-- ROADMAP -->
## Roadmap

- [ ] Scheduling / Provider System
- [ ] Undo System
- [ ] Moving Tasks / Groups
- [ ] Notification System

See the [open issues](https://github.com/suri-codes/TARS/issues) for a full list of planned features.

<!-- CONTRIBUTING -->
## Contributing

I feel as though the project is still in its early stages, so I'm not too big on
outside contributions. As I mentioned earlier, this project is highly
opinionated and a learning experience for me, so I want to take the time to
steer the ship where I want to take it. On the other hand, if you have a cool
idea you want to share, don't hesitate to make an issue, or even better, reach
out!

My policy on contributions will likely change once I deem the project 1.0'd, thank you for understanding.

<!-- LICENSE -->
## License

Distributed under the project_license. See `LICENSE` for more information.


<!-- CONTACT -->
## Contact

Email: suri312006@gmail.com

Website: www.suri.codes

Project Link: [https://github.com/suri-codes/TARS](https://github.com/suri-codes/TARS)


-- MARKDOWN LINKS & IMAGES -->
<!-- https://www.markdownguide.org/basic-syntax/#reference-style-links -->
[contributors-shield]: https://img.shields.io/github/contributors/suri-codes/TARS.svg?style=for-the-badge
[contributors-url]: https://github.com/suri-codes/TARS/graphs/contributors
[forks-shield]: https://img.shields.io/github/forks/suri-codes/TARS.svg?style=for-the-badge
[forks-url]: https://github.com/suri-codes/TARS/network/members
[stars-shield]: https://img.shields.io/github/stars/suri-codes/TARS.svg?style=for-the-badge
[stars-url]: https://github.com/suri-codes/TARS/stargazers
[issues-shield]: https://img.shields.io/github/issues/suri-codes/TARS.svg?style=for-the-badge
[issues-url]: https://github.com/suri-codes/TARS/issues
[license-shield]: https://img.shields.io/github/license/suri-codes/TARS.svg?style=for-the-badge
[license-url]: https://github.com/suri-codes/TARS/blob/master/LICENSE.txt
[linkedin-shield]: https://img.shields.io/badge/-LinkedIn-black.svg?style=for-the-badge&logo=linkedin&colorB=555
[linkedin-url]: https://linkedin.com/in/linkedin_username
[product-screenshot]: https://suri.codes/TARS_DEMO.png
<!-- Shields.io badges. You can a comprehensive list with many more badges at: https://github.com/inttter/md-badges -->
[Next.js]: https://img.shields.io/badge/next.js-000000?style=for-the-badge&logo=nextdotjs&logoColor=white
[Next-url]: https://nextjs.org/
[React.js]: https://img.shields.io/badge/React-20232A?style=for-the-badge&logo=react&logoColor=61DAFB
[React-url]: https://reactjs.org/
[Vue.js]: https://img.shields.io/badge/Vue.js-35495E?style=for-the-badge&logo=vuedotjs&logoColor=4FC08D
[Vue-url]: https://vuejs.org/
[Angular.io]: https://img.shields.io/badge/Angular-DD0031?style=for-the-badge&logo=angular&logoColor=white
[Angular-url]: https://angular.io/
[Svelte.dev]: https://img.shields.io/badge/Svelte-4A4A55?style=for-the-badge&logo=svelte&logoColor=FF3E00
[Svelte-url]: https://svelte.dev/
[Laravel.com]: https://img.shields.io/badge/Laravel-FF2D20?style=for-the-badge&logo=laravel&logoColor=white
[Laravel-url]: https://laravel.com
[Bootstrap.com]: https://img.shields.io/badge/Bootstrap-563D7C?style=for-the-badge&logo=bootstrap&logoColor=white
[Bootstrap-url]: https://getbootstrap.com
[JQuery.com]: https://img.shields.io/badge/jQuery-0769AD?style=for-the-badge&logo=jquery&logoColor=white
[JQuery-url]: https://jquery.com 
