pipeline {
    agent {
        docker {
            image 'magnonellie/amethyst-dependencies:stable'
            label 'docker'
        } 
    }
    stages {
        stage('fmt') {
            steps {
                sh 'cargo fmt --all -- --check'
            }
        }
        stage('test') {
            steps {
                sh 'cargo test --all'
            }
        }
    }
}
