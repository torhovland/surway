/// <reference types="cypress" />

describe('Surway React', () => {
  it('Loads and displays the map', () => {
    // Fake geolocation to Barcelona
    cy.visit('/', {
      onBeforeLoad(win: any) {
        cy.stub(win.navigator.geolocation, 'watchPosition').callsFake(
          (success: any, _error: any, _opts: any) => {
            success({
              coords: {
                latitude: 41.39,
                longitude: 2.16,
                altitude: null,
                accuracy: 5,
                altitudeAccuracy: null,
                heading: null,
                speed: null,
              },
              timestamp: Date.now(),
            });
            return 1;
          }
        );
      },
    });

    // Map should be visible
    cy.get('#map').should('be.visible');
    cy.get('.leaflet-container').should('exist');

    // Notes button should be visible
    cy.contains('Notes').should('be.visible');
  });

  it('Opens notes modal and can create a note', () => {
    cy.visit('/', {
      onBeforeLoad(win: any) {
        cy.stub(win.navigator.geolocation, 'watchPosition').callsFake(
          (success: any) => {
            success({
              coords: {
                latitude: 41.39,
                longitude: 2.16,
                accuracy: 5,
              },
              timestamp: Date.now(),
            });
            return 1;
          }
        );
      },
    });

    // Click Notes button
    cy.contains('Notes').click();

    // Modal should be visible
    cy.get('.modal-overlay').should('be.visible');
    cy.contains('Take a note').click();

    // Type a note
    cy.get('textarea').type('Cypress test note');

    // Save
    cy.contains('Save').click();

    // Verify note appears in modal
    cy.contains('Cypress test note').should('be.visible');
  });
});