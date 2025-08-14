use anyhow::{Context, Result};
use std::process::Command;
use tracing::{info, error};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Component {
    Server,
    Operator,
    Host,
    All,
}

impl std::str::FromStr for Component {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "server" => Ok(Component::Server),
            "operator" => Ok(Component::Operator),
            "host" => Ok(Component::Host),
            "all" => Ok(Component::All),
            _ => Err(format!("Unknown component: {}. Valid options: server, operator, host, all", s)),
        }
    }
}

pub struct ImageBuilder {
    tag: String,
    no_cache: bool,
}

impl ImageBuilder {
    pub fn new(tag: String, no_cache: bool) -> Self {
        Self { tag, no_cache }
    }

    pub async fn build(&self, components: Vec<Component>) -> Result<()> {
        info!("Starting Docker image build process");

        let components_to_build = if components.contains(&Component::All) {
            vec![Component::Server, Component::Operator, Component::Host]
        } else {
            components
        };

        for component in components_to_build {
            self.build_component(component).await?;
        }

        info!("All images built successfully!");
        self.list_images().await?;
        
        Ok(())
    }

    async fn build_component(&self, component: Component) -> Result<()> {
        let (dockerfile, image_name) = match component {
            Component::Server => ("Dockerfile.server", "raworc-server"),
            Component::Operator => ("Dockerfile.operator", "raworc-operator"),
            Component::Host => ("Dockerfile.host", "raworc-host"),
            Component::All => unreachable!(),
        };

        let full_image_name = format!("{}:{}", image_name, self.tag);
        info!("Building {} image...", full_image_name);

        let mut cmd = Command::new("docker");
        cmd.arg("build")
            .arg("-f")
            .arg(dockerfile)
            .arg("-t")
            .arg(&full_image_name);

        if self.no_cache {
            cmd.arg("--no-cache");
        }

        cmd.arg(".");

        let output = cmd
            .output()
            .context(format!("Failed to execute docker build for {}", image_name))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("Docker build failed for {}: {}", image_name, stderr);
            return Err(anyhow::anyhow!(
                "Failed to build {} image: {}",
                image_name,
                stderr
            ));
        }

        info!("Successfully built {} image", full_image_name);
        Ok(())
    }

    async fn list_images(&self) -> Result<()> {
        info!("Raworc Docker images:");
        
        let output = Command::new("docker")
            .args(&["images", "--filter", "reference=raworc-*"])
            .output()
            .context("Failed to list Docker images")?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            println!("{}", stdout);
        }

        Ok(())
    }
}

pub async fn run(
    components: Vec<String>,
    tag: String,
    no_cache: bool,
    push: bool,
    registry: Option<String>,
) -> Result<()> {
    // Parse components
    let mut parsed_components = Vec::new();
    for comp_str in components {
        let component = comp_str.parse::<Component>()
            .map_err(|e| anyhow::anyhow!(e))?;
        parsed_components.push(component);
    }

    // If no components specified, build all
    if parsed_components.is_empty() {
        parsed_components.push(Component::All);
    }

    // Build images
    let builder = ImageBuilder::new(tag.clone(), no_cache);
    builder.build(parsed_components.clone()).await?;

    // Push to registry if requested
    if push {
        push_images(parsed_components, tag, registry).await?;
    }

    Ok(())
}

async fn push_images(
    components: Vec<Component>,
    tag: String,
    registry: Option<String>,
) -> Result<()> {
    info!("Pushing images to registry");

    let components_to_push = if components.contains(&Component::All) {
        vec![Component::Server, Component::Operator, Component::Host]
    } else {
        components
    };

    for component in components_to_push {
        let image_name = match component {
            Component::Server => "raworc-server",
            Component::Operator => "raworc-operator",
            Component::Host => "raworc-host",
            Component::All => unreachable!(),
        };

        let source_image = format!("{}:{}", image_name, tag);
        
        let target_image = if let Some(ref reg) = registry {
            let target = format!("{}/{}:{}", reg, image_name, tag);
            
            // Tag image for registry
            info!("Tagging {} as {}", source_image, target);
            let tag_output = Command::new("docker")
                .args(&["tag", &source_image, &target])
                .output()
                .context("Failed to tag image")?;

            if !tag_output.status.success() {
                let stderr = String::from_utf8_lossy(&tag_output.stderr);
                return Err(anyhow::anyhow!("Failed to tag image: {}", stderr));
            }
            
            target
        } else {
            source_image.clone()
        };

        // Push image
        info!("Pushing {}", target_image);
        let push_output = Command::new("docker")
            .args(&["push", &target_image])
            .output()
            .context("Failed to push image")?;

        if !push_output.status.success() {
            let stderr = String::from_utf8_lossy(&push_output.stderr);
            return Err(anyhow::anyhow!("Failed to push image: {}", stderr));
        }

        info!("Successfully pushed {}", target_image);
    }

    Ok(())
}