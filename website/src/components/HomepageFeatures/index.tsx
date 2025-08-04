import type {ReactNode} from 'react';
import clsx from 'clsx';
import Heading from '@theme/Heading';
import styles from './styles.module.css';

type FeatureItem = {
  title: string;
  icon: string;
  description: ReactNode;
};

const FeatureList: FeatureItem[] = [
  {
    title: 'Session-Based Architecture',
    icon: '🚀',
    description: (
      <>
        Work organized into discrete, manageable sessions with automatic state preservation.
        Each session runs in its own containerized environment with complete isolation.
      </>
    ),
  },
  {
    title: 'Kubernetes Native',
    icon: '☸️',
    description: (
      <>
        Built on Kubernetes for scalable container orchestration. Deploy across any cloud
        or on-premises infrastructure with automatic scaling and high availability.
      </>
    ),
  },
  {
    title: 'Persistent State',
    icon: '💾',
    description: (
      <>
        Never lose your work. Kubernetes persistent volumes ensure state preservation
        across session restarts, enabling seamless continuation and remixing.
      </>
    ),
  },
  {
    title: 'Flexible Agent System',
    icon: '🤖',
    description: (
      <>
        Deploy multiple AI agents per session with dynamic loading. Support for built-in
        agents and external integrations with customizable guardrails.
      </>
    ),
  },
  {
    title: 'RBAC Security',
    icon: '🔐',
    description: (
      <>
        Fine-grained role-based access control inspired by Kubernetes. Manage permissions
        at resource and verb level with namespace isolation.
      </>
    ),
  },
  {
    title: 'REST API',
    icon: '📡',
    description: (
      <>
        Comprehensive REST API with OpenAPI documentation. Interactive CLI and planned
        SDKs for seamless integration into your workflows.
      </>
    ),
  },
];

function Feature({title, icon, description}: FeatureItem) {
  return (
    <div className={clsx('col col--4')}>
      <div className="text--center">
        <div className={styles.featureIcon}>{icon}</div>
      </div>
      <div className="text--center padding-horiz--md">
        <Heading as="h3">{title}</Heading>
        <p>{description}</p>
      </div>
    </div>
  );
}

export default function HomepageFeatures(): ReactNode {
  return (
    <section className={styles.features}>
      <div className="container">
        <div className="row">
          {FeatureList.map((props, idx) => (
            <Feature key={idx} {...props} />
          ))}
        </div>
      </div>
    </section>
  );
}
