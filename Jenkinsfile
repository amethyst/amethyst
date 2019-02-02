pipeline {
    agent {
        docker {
            image 'magnonellie/amethyst-dependencies:stable'
            label 'linux'
        } 
    }
    stages {
        stage('build') {
            steps {
                sh 'cargo build --all'
            }
        }
    }
}
