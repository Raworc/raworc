import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

// This runs in Node.js - Don't use client-side code here (browser APIs, JSX...)

/**
 * Creating a sidebar enables you to:
 - create an ordered group of docs
 - render a sidebar for each doc of that group
 - provide next/previous navigation

 The sidebars can be generated from the filesystem, or explicitly defined here.

 Create as many sidebars as you want.
 */
const sidebars: SidebarsConfig = {
  // Main documentation sidebar
  tutorialSidebar: [
    {
      type: 'doc',
      id: 'intro',
      label: 'Introduction',
    },
    {
      type: 'category',
      label: 'Getting Started',
      link: {
        type: 'generated-index',
        title: 'Getting Started',
        description: 'Learn how to install and use Raworc',
      },
      collapsed: false,
      items: [
        'getting-started/quickstart',
        'getting-started/installation',
      ],
    },
    {
      type: 'category',
      label: 'Core Concepts',
      link: {
        type: 'generated-index',
        title: 'Core Concepts',
        description: 'Understand the fundamental concepts of Raworc',
      },
      items: [
        'concepts/architecture',
        'concepts/sessions',
      ],
    },
    {
      type: 'category',
      label: 'User Guides',
      link: {
        type: 'generated-index',
        title: 'User Guides',
        description: 'Learn how to use Raworc effectively',
      },
      items: [
        'guides/cli-examples',
      ],
    },
    {
      type: 'category',
      label: 'Administrator Guide',
      link: {
        type: 'generated-index',
        title: 'Administrator Guide',
        description: 'Configure and manage your Raworc installation',
      },
      items: [
        'admin/configuration',
        'admin/rbac',
      ],
    },
    {
      type: 'category',
      label: 'API Reference',
      link: {
        type: 'generated-index',
        title: 'API Reference',
        description: 'Complete API documentation for Raworc',
      },
      items: [
        'api/overview',
        'api/rest-api',
      ],
    },
  ],
};

export default sidebars;
