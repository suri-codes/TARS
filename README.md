
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

  <p align="center">
    <br />
    <a href="https://github.com/suri-codes/TARS"><strong>Explore the docs »</strong></a>
    <br />
    <br />
    <a href="https://github.com/suri-codes/TARS">View Demo</a>
    &middot;
    <a href="https://github.com/suri-codes/TARS/issues/new?labels=bug&template=bug-report---.md">Report Bug</a>
    &middot;
    <a href="https://github.com/suri-codes/TARS/issues/new?labels=enhancement&template=feature-request---.md">Request Feature</a>
  </p>
</div>

<!-- ABOUT THE PROJECT -->
## About The Project

[![TARS Tui][product-screenshot]](https://suri.codes/TARS_DEMO.png)

I've been thining about writing highly personal solutions that solve my problems
exactly rather than being satisfied by off-the-shelf tools.  I'm the kind of
person to work on 5 different things at the same time, and keeping track of what
I'm trying to do with each project while also balancing school assignments is a
chore. So, I decided to make some software that would help me keep track of
short-term and longterm goals while also being quick and easy to use!

<!-- GETTING STARTED -->
## Getting Started

### Prerequisites
This project only requires rust, preferably the latest release.

### Installation

I plan to distribute prebuilt binaries in the future but for now you can do the following to build and install locally.

1. Clone the repo

   ```sh
   git clone https://github.com/suri-codes/TARS.git
   ```
2. Run the Daemon
   ```sh
   cargo run --bin tars-daemon
   ```
  If you want to use TARS long-term, I reccomend building the binary and adding this to your startup script. The TARS-TUI will not start without the daemon running. 
3. Run the TUI
   ```sh
  cargo run --bin tars-tui
   ```
  You can also alias this in your shell as `tars`.

<!-- USAGE EXAMPLES -->
## Usage

Use this space to show useful examples of how a project can be used. Additional screenshots, code examples and demos work well in this space. You may also link to more resources.

_For more examples, please refer to the [Documentation](https://example.com)_


<!-- ROADMAP -->
## Roadmap

- [ ] Feature 1
- [ ] Feature 2
- [ ] Feature 3
    - [ ] Nested Feature

See the [open issues](https://github.com/suri-codes/TARS/issues) for a full list of proposed features (and known issues).

<!-- CONTRIBUTING -->
## Contributing

// Not too big on contributions since the project is so early but if you really want to help out please make a PR and I can take a look

Contributions are what make the open source community such an amazing place to learn, inspire, and create. Any contributions you make are **greatly appreciated**.

If you have a suggestion that would make this better, please fork the repo and create a pull request. You can also simply open an issue with the tag "enhancement".
Don't forget to give the project a star! Thanks again!

1. Fork the Project
5. Open a Pull Request

<p align="right">(<a href="#readme-top">back to top</a>)</p>

### Top contributors:

<a href="https://github.com/suri-codes/TARS/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=suri-codes/TARS" alt="contrib.rocks image" />
</a>



<!-- LICENSE -->
## License

Distributed under the project_license. See `LICENSE` for more information.

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- CONTACT -->
## Contact

maybe my socials


Project Link: [https://github.com/suri-codes/TARS](https://github.com/suri-codes/TARS)

<p align="right">(<a href="#readme-top">back to top</a>)</p>

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
