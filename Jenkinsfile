pipeline {
	agent {
		label 'k8s-agent'
	}
	environment {
		GIT_REPO = 'git@github.com:mendinuetaj/mega-uploader-auth.git'
		GIT_CREDENTIALS = 'github'
		DOCKER_IMAGE = 'mega-uploader-auth'
		DOCKER_TAG = "${env.GIT_TAG?:'latest'}"
	}
	stages {
		stage('Checkout') {
			steps {
				echo "Checking out..."
				checkout([
					$class: 'GitSCM',
					branches: [[name: "*/${env.GIT_TAG?:'master'}"]],
					userRemoteConfigs: [[
						url: env.GIT_REPO,
						credentialsId: env.GIT_CREDENTIALS
					]]
				])
			}
		}
		stage('Build Docker Image') {
			steps {
				container('docker') {
					script {
						echo "Building Docker image..."
						sh "docker build -t ${DOCKER_IMAGE}:${DOCKER_TAG} ."
					}
				}
			}
		}
	}
}