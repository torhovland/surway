/// <reference types="cypress" />

export const setFakePosition = position => {
  Cypress.automation("remote:debugger:protocol", {
    command: "Emulation.setGeolocationOverride",
    params: {
      latitude: position.latitude,
      longitude: position.longitude,
      accuracy: 50
    }
  })
};

context('Surway', () => {
  beforeEach(() => {
    cy.visit('/', {
      onBeforeLoad() {
        setFakePosition({ latitude: 41.38879, longitude: 2.15899 });
      },
    })
  })

  describe('Initial way', () => {
    it('should show the nearest way', () => {
      cy.contains('footway = sidewalk');
    })
  })

  describe('Note-taking', () => {
    it('can record a note', () => {
      cy.contains('Take a note').click();
      cy.get('textarea').type("Foobar");
      cy.contains('Save').click().should(() => {
        expect(localStorage.getItem('notes')).to.contain('Foobar');
      })
    })
  })
})
