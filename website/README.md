# Raworc Documentation Website

This is the official documentation website for Raworc.

## Installation

```bash
npm install
# or
yarn
```

## Local Development

```bash
npm start
# or
yarn start
```

This command starts a local development server and opens up a browser window. Most changes are reflected live without having to restart the server.

## Build

```bash
npm run build
# or
yarn build
```

This command generates static content into the `build` directory and can be served using any static contents hosting service.

## Deployment

### Using GitHub Pages:

```bash
GIT_USER=<Your GitHub username> npm run deploy
# or
USE_SSH=true yarn deploy
```

### Using other hosting services:

1. Build the site: `npm run build`
2. Deploy the `build` folder to your hosting service

## Documentation Structure

- `/docs` - Main documentation content
  - `/getting-started` - Installation and quick start guides
  - `/concepts` - Core concepts and architecture
  - `/guides` - User guides and tutorials
  - `/admin` - Administrator documentation
  - `/api` - API reference
- `/blog` - Blog posts and announcements
- `/src` - Website source code
  - `/components` - React components
  - `/pages` - Additional pages
  - `/css` - Custom styles

## Key Features

- 📚 Comprehensive documentation for Raworc
- 🔍 Algolia search integration (requires configuration)
- 📱 Mobile-responsive design
- 🌙 Dark mode support
- 📊 API reference with examples
- 🚀 Quick start guides

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test locally with `npm start`
5. Submit a pull request

## Algolia Search

The site is configured to use Algolia DocSearch. To enable search:

1. Apply for [DocSearch](https://docsearch.algolia.com/)
2. Update the Algolia configuration in `docusaurus.config.ts` with your credentials

## License

This documentation is licensed under the same license as the Raworc project.
